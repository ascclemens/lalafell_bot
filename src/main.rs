#![feature(mpsc_select, box_syntax)]

extern crate discord;
extern crate xivdb;
extern crate dotenv;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate ctrlc;
extern crate chrono;
#[macro_use]
extern crate log;
extern crate simplelog;

mod database;
mod listeners;
mod tasks;

use database::*;
use listeners::*;
use tasks::*;

use discord::{Discord, State};
use discord::model::{Message, Channel, LiveServer, UserId, Role, ReactionEmoji, ChannelId, MessageId};
use discord::model::permissions;

use xivdb::XivDb;
use xivdb::error::*;

use chrono::prelude::*;

use simplelog::{TermLogger, LogLevel, LogLevelFilter, Config};

use std::collections::HashMap;
use std::env::var;
use std::fs::{File, OpenOptions};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver};

macro_rules! opt_or {
  ($expr: expr, $or: expr) => {{
    match $expr {
      Some(x) => x,
      None => $or
    }
  }}
}

fn main() {
  let mut config = Config::default();
  config.target = Some(LogLevel::Error);
  TermLogger::init(LogLevelFilter::Info, config).unwrap();

  dotenv::dotenv().ok();

  let bot_token = match var("LB_DISCORD_BOT_TOKEN") {
    Ok(t) => t,
    Err(_) => {
      error!("No bot token was specified in .env");
      return;
    }
  };
  let database_location = match var("LB_DATABASE_LOCATION") {
    Ok(t) => t,
    Err(_) => {
      error!("No database location was specified in .env");
      return;
    }
  };

  let bot = match LalafellBot::new(&bot_token, &database_location) {
    Ok(b) => Arc::new(b),
    Err(e) => {
      error!("could not create bot: {}", e.iter().map(|err| err.to_string()).collect::<Vec<_>>().join("\n"));
      return;
    }
  };

  {
    let mut listeners = bot.listeners.lock().unwrap();
    // listeners.push(box EventDebugger);
    listeners.push(box CommandListener::new(bot.clone()));
  }

  let (loop_cancel_tx, loop_cancel_rx) = channel();

  ctrlc::set_handler(move || {
    info!("Stopping main loop");
    loop_cancel_tx.send(()).unwrap();
  }).expect("could not set interrupt handler");

  info!("Starting tasks");

  let task_manager = TaskManager::new(bot.clone());
  task_manager.start_task(DatabaseSaveTask::new(&database_location));
  task_manager.start_task(AutoTagTask::new());
  task_manager.start_task(DeleteAllMessagesTask::new(
    ChannelId(307359970096185345),
    30,
    vec![MessageId(307367821506117642)]
  ));

  info!("Spinning up bot");
  let thread_bot = bot.clone();
  std::thread::spawn(move || {
    if let Err(e) = thread_bot.start_loop(loop_cancel_rx) {
      error!("could not start bot loop: {}", e);
    }
  }).join().unwrap();
  info!("Saving database");
  bot.save_database(&database_location).unwrap();
  info!("Exiting");
}

pub struct LalafellBot {
  pub discord: Discord,
  pub xivdb: XivDb,
  pub state: Mutex<Option<State>>,
  pub database: Mutex<Database>,
  pub listeners: Mutex<Vec<Box<ReceivesEvents + Send>>>
}

impl Drop for LalafellBot {
  fn drop(&mut self) {
    self.save_database(&var("LB_DATABASE_LOCATION").unwrap()).unwrap()
  }
}

impl LalafellBot {
  fn new(bot_token: &str, database_location: &str) -> Result<LalafellBot> {
    let discord = Discord::from_bot_token(bot_token).chain_err(|| "could not start discord from bot token")?;
    let mut database = LalafellBot::load_database(database_location)?;
    database.last_saved = UTC::now().timestamp();
    Ok(LalafellBot {
      discord: discord,
      xivdb: XivDb::default(),
      state: Mutex::new(None),
      database: Mutex::new(database),
      listeners: Mutex::new(Vec::new())
    })
  }

  fn load_database(location: &str) -> Result<Database> {
    if !Path::new(location).exists() {
      return Ok(Database::default());
    }
    let f = File::open(location).chain_err(|| "could not open or create database file")?;
    serde_json::from_reader(f).chain_err(|| "could not deserialize database")
  }

  fn save_database(&self, location: &str) -> Result<()> {
    let f = OpenOptions::new()
      .write(true)
      .truncate(true)
      .create(true)
      .open(location)
      .chain_err(|| "could not open or create database file")?;
    serde_json::to_writer(f, &self.database).chain_err(|| "could not serialize database")
  }

  fn start_loop(&self, loop_cancel: Receiver<()>) -> Result<()> {
    let (mut connection, ready) = self.discord.connect().chain_err(|| "could not connect to discord")?;
    let state = State::new(ready);
    *self.state.lock().unwrap() = Some(state);
    connection.set_game_name("with other Lalafell.".to_string());
    let (event_channel_tx, event_channel_rx) = channel();
    std::thread::spawn(move || {
      loop {
        if let Err(e) = event_channel_tx.send(connection.recv_event()) {
          error!("error sending event: {}", e);
        }
      }
    });
    info!("Starting main loop");
    loop {
      let event = select! {
        _ = loop_cancel.recv() => break,
        event = event_channel_rx.recv() => event
      };
      let event = match event {
        Ok(Ok(e)) => e,
        Ok(Err(e)) => {
          warn!("could not receive event from select: {}", e);
          continue;
        },
        Err(e) => {
          warn!("could not receive discord event: {}", e);
          continue;
        }
      };
      {
        let mut state_option = self.state.lock().unwrap();
        let mut state = state_option.as_mut().unwrap();
        state.update(&event);
      }
      {
        let listeners = self.listeners.lock().unwrap();
        for listener in listeners.iter() {
          listener.receive(&event);
        }
      }
    }
    info!("Main loop stopped");
    Ok(())
  }

  fn check_command(&self, message: &Message) {
    let parts: Vec<&str> = message.content.split_whitespace().collect();
    if parts.is_empty() {
      return;
    }
    let first = parts[0];
    if !first.starts_with('!') {
      return;
    }
    let command_name = &first[1..].to_lowercase();
    let params = &parts[1..];
    let result = match command_name.as_str() {
      "race" => self.race_command(message, params),
      "autotag" => self.autotag_command(message, params),
      "tag" => self.tag_command(message, params),
      "viewtag" => self.viewtag_command(message, params),
      _ => return
    };
    if let Err(e) = result {
      self.discord.send_embed(message.channel_id, "",
        |e| e.description("An internal error happened while processing this command.")).ok();
      for err in e.iter() {
        error!("error: {:#?}", err);
      }
    }
  }

  fn race_command(&self, message: &Message, params: &[&str]) -> Result<bool> {
    if params.len() < 3 {
      return Ok(false);
    }
    let server = params[0];
    let name = params[1..].join(" ");
    let mut params = HashMap::new();
    params.insert(String::from("one"), String::from("characters"));
    params.insert(String::from("strict"), String::from("on"));
    params.insert(String::from("server|et"), server.to_string());
    let res = self.xivdb.search(name.clone(), params).chain_err(|| "could not search XIVDB")?;
    let search_chars = match res.characters {
      Some(c) => c.results,
      None => return Err("no characters field in search result".into())
    };
    if search_chars.is_empty() {
      self.discord.send_embed(
        message.channel_id, "",
        |f| f.description(&format!("Could not find any character by the name {}.", name))).chain_err(|| "could not send embed")?;
      return Ok(true);
    }
    let character = self.xivdb.character(search_chars[0]["id"].as_u64().unwrap()).unwrap();
    if character.name.to_lowercase() != name.to_lowercase() {
      self.discord.send_embed(
        message.channel_id, "",
        |f| f.description(&format!("Could not find any character by the name {}.", name))).chain_err(|| "could not send embed")?;
      return Ok(true);
    }
    self.discord.send_embed(
      message.channel_id, "",
      |f| f.description(&format!("{} ({})", character.data.race, character.data.clan))).chain_err(|| "could not send embed")?;
    Ok(true)
  }

  fn viewtag_command(&self, message: &Message, params: &[&str]) -> Result<bool> {
    if params.is_empty() {
      return Ok(false);
    }
    let channel = self.discord.get_channel(message.channel_id).chain_err(|| "could not get channel for message")?;
    let server_id = match channel {
      Channel::Public(c) => c.server_id,
      _ => return Err("channel was not public".into())
    };
    let who = params[0];
    let who = if !who.starts_with("<@") && !who.ends_with('>') && message.mentions.len() != 1 {
      match who.parse::<u64>() {
        Ok(n) => UserId(n),
        Err(_) => return Ok(false)
      }
    } else {
      message.mentions[0].id
    };

    let user = {
      let database = self.database.lock().unwrap();
      database.autotags.users.iter().find(|u| u.user_id == who.0 && u.server_id == server_id.0).cloned()
    };

    let msg = match user {
      Some(u) => format!("{} is {} on {}.", who.mention(), u.character, u.server),
      None => format!("{} is not tagged.", who.mention())
    };
    self.discord.send_embed(message.channel_id, "", |e| e.description(&msg)).chain_err(|| "could not send embed")?;
    Ok(true)
  }

  fn autotag_command(&self, message: &Message, params: &[&str]) -> Result<bool> {
    let channel = self.discord.get_channel(message.channel_id).chain_err(|| "could not get channel for message")?;
    let server_id = match channel {
      Channel::Public(c) => c.server_id,
      _ => return Err("channel was not public".into())
    };
    let mut state_option = self.state.lock().unwrap();
    let state = state_option.as_mut().unwrap();
    let server = match state.servers().iter().find(|x| x.id == server_id) {
      Some(s) => s,
      None => return Err("could not find server for channel".into())
    };

    if params.len() < 3 {
      return Ok(false);
    }

    let ff_server = params[0];
    let name = params[1..].join(" ");

    let (msg, emoji) = match self.tag(message.author.id, server, ff_server, &name)? {
      Some(error) => (Some(error), ReactionEmoji::Unicode(String::from("\u{274c}"))),
      None => (None, ReactionEmoji::Unicode(String::from("\u{2705}")))
    };
    if let Some(msg) = msg {
      self.discord.send_embed(message.channel_id, "", |f| f.description(&msg)).chain_err(|| "could not send embed")?;
    }
    self.discord.add_reaction(message.channel_id, message.id, emoji).chain_err(|| "could not add reaction")?;
    Ok(true)
  }

  fn tag_command(&self, message: &Message, params: &[&str]) -> Result<bool> {
    let channel = self.discord.get_channel(message.channel_id).chain_err(|| "could not get channel for message")?;
    let server_id = match channel {
      Channel::Public(c) => c.server_id,
      _ => return Err("channel was not public".into())
    };
    let user = self.discord.get_member(server_id, message.author.id).chain_err(|| "could not get member for message")?;
    let mut state_option = self.state.lock().unwrap();
    let state = state_option.as_mut().unwrap();
    let server = match state.servers().iter().find(|x| x.id == server_id) {
      Some(s) => s,
      None => return Err("could not find server for channel".into())
    };
    if server.owner_id != message.author.id {
      let roles = &server.roles;
      let user_roles: Option<Vec<&Role>> = user.roles.iter()
        .map(|r| roles.iter().find(|z| z.id == *r))
        .collect();
      match user_roles {
        Some(ur) => {
          let can_manage_roles = ur.iter()
            .any(|r| r.permissions.contains(permissions::MANAGE_ROLES));
          if !can_manage_roles {
            return Ok(false);
          }
        },
        None => return Ok(false)
      }
    }

    if params.len() < 3 {
      return Ok(false);
    }

    let who = params[0];
    let who = if !who.starts_with("<@") && !who.ends_with('>') && message.mentions.len() != 1 {
      match who.parse::<u64>() {
        Ok(n) => UserId(n),
        Err(_) => return Ok(false)
      }
    } else {
      message.mentions[0].id
    };
    let ff_server = params[1];
    let name = params[2..].join(" ");

    let (msg, emoji) = match self.tag(who, server, ff_server, &name)? {
      Some(error) => (Some(error), ReactionEmoji::Unicode(String::from("\u{274c}"))),
      None => (None, ReactionEmoji::Unicode(String::from("\u{2705}")))
    };
    if let Some(msg) = msg {
      self.discord.send_embed(message.channel_id, "", |f| f.description(&msg)).chain_err(|| "could not send embed")?;
    }
    self.discord.add_reaction(message.channel_id, message.id, emoji).chain_err(|| "could not add reaction")?;
    Ok(true)
  }

  fn tag(&self, who: UserId, on: &LiveServer, server: &str, character_name: &str) -> Result<Option<String>> {
    let mut params = HashMap::new();
    params.insert(String::from("one"), String::from("characters"));
    params.insert(String::from("strict"), String::from("on"));
    params.insert(String::from("server|et"), server.to_string());

    let res = self.xivdb.search(character_name.to_string(), params).chain_err(|| "could not query XIVDB")?;

    let search_chars = res.characters.unwrap().results;
    if search_chars.is_empty() {
      return Ok(Some(format!("Could not find any character by the name {} on {}.", character_name, server)));
    }

    let char_id = match search_chars[0]["id"].as_u64() {
      Some(u) => u,
      None => return Err("character ID was not a u64".into())
    };
    let character = self.xivdb.character(char_id).chain_err(|| "could not look up character")?;

    if character.name.to_lowercase() != character_name.to_lowercase() {
      return Ok(Some(format!("Could not find any character by the name {} on {}.", character_name, server)));
    }

    self.database.lock().unwrap().autotags.update_or_remove(AutotagUser::new(
      who.0,
      on.id.0,
      &character.name,
      &character.server
    ));

    let roles = &on.roles;
    let mut add_roles = Vec::with_capacity(2);
    if let Some(r) = roles.iter().find(|x| x.name.to_lowercase() == character.data.race.to_lowercase()) {
      add_roles.push(r.id);
    }
    if let Some(r) = roles.iter().find(|x| x.name.to_lowercase() == character.data.gender.to_lowercase()) {
      add_roles.push(r.id);
    }
    if let Some(r) = roles.iter().find(|x| x.name.to_lowercase() == character.server.to_lowercase()) {
      add_roles.push(r.id);
    }

    self.discord.edit_member_roles(on.id, who, &add_roles).chain_err(|| "could not add roles")?;
    // cannot edit nickname of server owners
    self.discord.edit_member(on.id, who, |e| e.nickname(&character.name)).ok();
    Ok(None)
  }
}
