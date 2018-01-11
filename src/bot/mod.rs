use Environment;

use error::*;
use config::Config;

use xivdb::XivDb;

use serenity::client::Client;
use serenity::prelude::RwLock;

use diesel::Connection;
use diesel::pg::PgConnection;

use std::env;
use std::sync::Arc;

mod creation;

pub mod data;

pub use self::creation::config;

thread_local! {
  pub static CONNECTION: PgConnection = PgConnection::establish(&env::var("LB_DATABASE_LOCATION").unwrap()).unwrap();
}

pub use self::creation::{create_bot, Handler};

pub struct LalafellBot {
  pub discord: Client,
  pub env: Arc<BotEnv>
}

pub struct BotEnv {
  pub environment: Environment,
  pub config: RwLock<Config>,
  pub xivdb: XivDb
}

impl LalafellBot {
  pub fn new(environment: Environment, config: Config) -> Result<LalafellBot> {
    let env = Arc::new(BotEnv {
      environment,
      config: RwLock::new(config),
      xivdb: XivDb
    });
    let client = Client::new(&env.environment.discord_bot_token, Handler::new(Arc::clone(&env)))?;
    Ok(LalafellBot {
      discord: client,
      env
    })
  }
}
