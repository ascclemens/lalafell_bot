use Environment;
use config::Config;

use lalafell::bot::Bot;
use lalafell::listeners::ReceivesEvents;

use xivdb::XivDb;
use error::*;

use discord::{Discord, State};

use diesel::Connection;
use diesel::sqlite::SqliteConnection;

use std::sync::RwLock;
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::env;

mod creation;

thread_local! {
  pub static CONNECTION: SqliteConnection = LalafellBot::database_connection(&env::var("LB_DATABASE_LOCATION").unwrap()).unwrap();
}

pub use self::creation::create_bot;

pub struct LalafellBot {
  pub environment: Environment,
  pub config: Config,
  pub discord: Discord,
  pub state: RwLock<Option<State>>,
  pub xivdb: XivDb,
  pub listeners: RwLock<Vec<Box<ReceivesEvents + Send + Sync>>>
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
    Ok(LalafellBot {
      environment: environment,
      config: config,
      discord: discord,
      state: RwLock::default(),
      xivdb: XivDb::default(),
      listeners: RwLock::default()
    })
  }

  pub fn database_connection(location: &str) -> Result<SqliteConnection> {
    SqliteConnection::establish(location)
      .chain_err(|| format!("could not connect to sqlite database at {}", location))
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
