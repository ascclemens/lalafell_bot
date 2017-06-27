use Environment;
use config::Config;
use database::Database;

use lalafell::bot::Bot;
use lalafell::listeners::ReceivesEvents;

use xivdb::XivDb;
use error::*;

use discord::{Discord, State};

use serde_json;

use chrono::prelude::*;

use std::fs::{OpenOptions, File};
use std::path::Path;
use std::sync::RwLock;
use std::sync::mpsc::{channel, Receiver};
use std::thread;

mod creation;

pub use self::creation::create_bot;

pub struct LalafellBot {
  pub environment: Environment,
  pub config: Config,
  pub discord: Discord,
  pub state: RwLock<Option<State>>,
  pub xivdb: XivDb,
  pub database: RwLock<Database>,
  pub listeners: RwLock<Vec<Box<ReceivesEvents + Send + Sync>>>
}

impl Drop for LalafellBot {
  fn drop(&mut self) {
    self.save_database(None).unwrap()
  }
}

impl Bot for LalafellBot {
  fn discord(&self) -> &Discord {
    &self.discord
  }

  fn discord_mut(&mut self) -> &mut Discord {
    &mut self.discord
  }

  fn state(&self) -> &RwLock<Option<State>> {
    &self.state
  }

  fn listeners(&self) -> &RwLock<Vec<Box<ReceivesEvents + Send + Sync>>> {
    &self.listeners
  }
}

impl LalafellBot {
  pub fn new(environment: Environment, config: Config) -> Result<LalafellBot> {
    let discord = Discord::from_bot_token(&environment.discord_bot_token).chain_err(|| "could not start discord from bot token")?;
    let mut database = LalafellBot::load_database(&environment.database_location)?;
    database.last_saved = Utc::now().timestamp();
    Ok(LalafellBot {
      environment: environment,
      config: config,
      discord: discord,
      state: RwLock::default(),
      xivdb: XivDb::default(),
      database: RwLock::new(database),
      listeners: RwLock::default()
    })
  }

  pub fn load_database(location: &str) -> Result<Database> {
    if !Path::new(location).exists() {
      return Ok(Database::default());
    }
    let f = File::open(location).chain_err(|| "could not open or create database file")?;
    serde_json::from_reader(f).chain_err(|| "could not deserialize database")
  }

  pub fn save_database(&self, location: Option<&str>) -> Result<()> {
    let f = OpenOptions::new()
      .write(true)
      .truncate(true)
      .create(true)
      .open(location.unwrap_or(&self.environment.database_location))
      .chain_err(|| "could not open or create database file")?;
    serde_json::to_writer(f, &self.database).chain_err(|| "could not serialize database")
  }

  pub fn start_loop(&self, loop_cancel: Receiver<()>) -> Result<()> {
    let (mut connection, ready) = self.discord.connect().chain_err(|| "could not connect to discord")?;
    *self.state.write().unwrap() = Some(State::new(ready));

    connection.set_game_name("with other Lalafell.".to_string());

    let (event_channel_tx, event_channel_rx) = channel();
    thread::spawn(move || {
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
        let mut state_option = self.state.write().unwrap();
        let mut state = state_option.as_mut().unwrap();
        state.update(&event);
      }
      {
        let listeners = self.listeners.read().unwrap();
        for listener in listeners.iter() {
          listener.receive(&event);
        }
      }
    }
    info!("Main loop stopped");
    Ok(())
  }
}
