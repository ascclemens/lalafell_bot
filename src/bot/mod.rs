use Environment;
use config::Config;

use xivdb::XivDb;
use error::*;

use serenity::client::Client;

use diesel::Connection;
use diesel::connection::SimpleConnection;
use diesel::sqlite::SqliteConnection;

use std::sync::Arc;
use std::env;

mod creation;

thread_local! {
  pub static CONNECTION: SqliteConnection = LalafellBot::database_connection(&env::var("LB_DATABASE_LOCATION").unwrap()).unwrap();
}

pub use self::creation::{create_bot, Handler};

pub struct LalafellBot {
  pub discord: Client,
  pub env: Arc<BotEnv>
}

pub struct BotEnv {
  pub environment: Environment,
  pub config: Config,
  pub xivdb: XivDb
}

impl LalafellBot {
  pub fn new(environment: Environment, config: Config) -> Result<LalafellBot> {
    let env = Arc::new(BotEnv {
      environment,
      config,
      xivdb: XivDb
    });
    let client = Client::new(&env.environment.discord_bot_token, Handler::new(env.clone()))?;
    Ok(LalafellBot {
      discord: client,
      env
    })
  }

  pub fn database_connection(location: &str) -> Result<SqliteConnection> {
    let connection = SqliteConnection::establish(location)
      .chain_err(|| format!("could not connect to sqlite database at {}", location))?;
    connection.batch_execute("PRAGMA foreign_keys = ON;").chain_err(|| "could not enable foreign keys")?;
    Ok(connection)
  }
}
