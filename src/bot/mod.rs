use Environment;

use error::Result as BotResult;
use config::Config;
use database::models::ToU64;

use lalafell::error::Result as LalafellResult;

use xivdb::XivDb;

use serenity::client::Client;
use serenity::prelude::RwLock;
use serenity::model::id::UserId;

use diesel::prelude::*;
use diesel::Connection;
use diesel::pg::PgConnection;

use std::env;
use std::sync::Arc;

mod creation;

pub use self::creation::data;
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
  pub fn new(environment: Environment, config: Config) -> BotResult<LalafellBot> {
    let env = Arc::new(BotEnv {
      environment,
      config: RwLock::new(config),
      xivdb: XivDb::default()
    });
    let client = Client::new(&env.environment.discord_bot_token, Handler::new(&env))?;
    Ok(LalafellBot {
      discord: client,
      env
    })
  }
}

pub fn is_administrator<U: Into<UserId>>(user: U) -> LalafellResult<bool> {
  use lalafell::error::ResultExt;
  let user_id = user.into();
  let number_matching: i64 = ::bot::CONNECTION.with(|c| {
    use database::schema::administrators::dsl;
    dsl::administrators
      .find(user_id.to_u64())
      .count()
      .get_result(c)
      .chain_err(|| "could not check administrators")
  })?;
  Ok(number_matching > 0)
}
