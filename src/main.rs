#![feature(mpsc_select, box_syntax, fnbox)]

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
extern crate hyper;
extern crate scraper;
extern crate uuid;
extern crate toml;

mod database;
mod listeners;
mod tasks;
mod commands;
mod lodestone;
mod config;

use database::*;
use listeners::*;
use tasks::*;
use config::Config;

use discord::{Discord, State};
use discord::model::{ChannelId, MessageId};

use xivdb::XivDb;
use xivdb::error::*;

use chrono::prelude::*;

use simplelog::{TermLogger, LogLevel, LogLevelFilter};

use std::env::var;
use std::fs::{File, OpenOptions};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver};
use std::io::Read;

// Add verification system with lodestone profile

macro_rules! opt_or {
  ($expr: expr, $or: expr) => {{
    match $expr {
      Some(x) => x,
      None => $or
    }
  }}
}

fn main() {
  {
    let mut config = simplelog::Config::default();
    config.target = Some(LogLevel::Error);
    TermLogger::init(LogLevelFilter::Info, config).unwrap();
  }

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
  let config_location = match var("LB_CONFIG_LOCATION") {
    Ok(t) => t,
    Err(_) => {
      error!("No config location was specified in .env");
      return;
    }
  };

  let config: Config = match File::open(config_location) {
    Ok(mut f) => {
      let mut content = String::new();
      f.read_to_string(&mut content).unwrap();
      toml::from_str(&content).unwrap()
    },
    Err(_) => Default::default()
  };

  let bot = match LalafellBot::new(config, &bot_token, &database_location) {
    Ok(b) => Arc::new(b),
    Err(e) => {
      error!("could not create bot: {}", e.iter().map(|err| err.to_string()).collect::<Vec<_>>().join("\n"));
      return;
    }
  };

  let mut command_listener = listeners::commands::CommandListener::new(bot.clone());
  command_listener.add_command(&["race"], box RaceCommand::new(bot.clone()));
  command_listener.add_command(&["tag"], box TagCommand::new(bot.clone()));
  command_listener.add_command(&["autotag"], box AutoTagCommand::new(bot.clone()));
  command_listener.add_command(&["viewtag"], box ViewTagCommand::new(bot.clone()));
  command_listener.add_command(&["updatetags"], box UpdateTagsCommand::new(bot.clone()));
  command_listener.add_command(&["savedatabase"], box SaveDatabaseCommand::new(bot.clone(), database_location.clone()));
  command_listener.add_command(&["verify"], box commands::verify::VerifyCommand::new(bot.clone()));

  {
    let mut listeners = bot.listeners.lock().unwrap();
    // listeners.push(box EventDebugger);
    listeners.push(box command_listener);
    listeners.push(box TagInstructions::new(bot.clone()));
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
    300,
    bot.config.delete_all_messages_task.except.iter().map(|id| MessageId(*id)).collect()
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
  pub config: Config,
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
  fn new(config: Config, bot_token: &str, database_location: &str) -> Result<LalafellBot> {
    let discord = Discord::from_bot_token(bot_token).chain_err(|| "could not start discord from bot token")?;
    let mut database = LalafellBot::load_database(database_location)?;
    database.last_saved = UTC::now().timestamp();
    Ok(LalafellBot {
      config: config,
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
}
