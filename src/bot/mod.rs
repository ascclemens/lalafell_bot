use crate::Environment;
use crate::error::Result as BotResult;
use crate::config::Config;
use crate::database::models::ToU64;

use lalafell::error::Result as LalafellResult;

use lodestone_api_client::LodestoneApi;

use serenity::{
  CacheAndHttp,
  cache::{Cache, CacheRwLock},
  client::Client,
  http::Http,
  model::id::UserId,
  prelude::RwLock,
};

use diesel::result;
use diesel::prelude::*;
use diesel::Connection;
use diesel::pg::PgConnection;

use std::{
  env,
  mem::MaybeUninit,
  sync::Arc,
};

mod creation;

pub use self::creation::data;
pub use self::creation::config;

// FIXME: do something nice than panic for these connections
//        the bot will stay up and reconnect, but panics are ugly and could be handled better
thread_local! {
  static CONNECTION: RwLock<PgConnection> = RwLock::new(connect_database().expect("could not connect to database"));
}

fn connect_database() -> Option<PgConnection> {
  env::var("LB_DATABASE_LOCATION").ok()
    .and_then(|loc| PgConnection::establish(&loc).ok())
}

pub fn with_connection<F, T>(f: F) -> QueryResult<T>
  where F: Fn(&PgConnection) -> QueryResult<T>
{
  with_connection_retries(f, 0)
}

fn with_connection_retries<F, T>(f: F, retries: usize) -> QueryResult<T>
  where F: Fn(&PgConnection) -> QueryResult<T>
{
  let res = CONNECTION.with(|c| f(&c.read()));
  match res {
    Ok(t) => Ok(t),
    Err(result::Error::DatabaseError(result::DatabaseErrorKind::UnableToSendCommand, _)) if retries < 3 => {
      CONNECTION.with(|c| {
        *c.write() = connect_database().expect("could not connect to database");
      });
      with_connection_retries(f, retries + 1)
    },
    Err(e) => Err(e)
  }
}

pub use self::creation::{create_bot, Handler};

pub struct LalafellBot {
  pub discord: Client,
  pub env: Arc<BotEnv>
}

pub struct BotEnv {
  pub environment: Environment,
  pub config: RwLock<Config>,
  pub lodestone: LodestoneApi,
  cah: RwLock<MaybeUninit<Arc<CacheAndHttp>>>,
}

impl BotEnv {
  fn cah(&self) -> Arc<CacheAndHttp> {
    let cah = self.cah.read();
    let cah = unsafe { cah.get_ref() };
    Arc::clone(cah)
  }

  pub fn http(&self) -> Arc<Http> {
    let cah = self.cah();
    Arc::clone(&cah.http)
  }

  pub fn cache(&self) -> Arc<RwLock<Cache>> {
    let cah = self.cah();
    Arc::clone(&cah.cache)
  }

  pub fn cache_lock(&self) -> CacheRwLock {
    self.cache().into()
  }
}

impl LalafellBot {
  pub fn new(environment: Environment, config: Config) -> BotResult<LalafellBot> {
    let env = Arc::new(BotEnv {
      lodestone: LodestoneApi::default(),
      config: RwLock::new(config),
      environment,
      cah: RwLock::new(MaybeUninit::uninit()),
    });
    let client = Client::new(&env.environment.discord_bot_token, Handler::new(&env))?;
    env.cah.write().write(Arc::clone(&client.cache_and_http));
    Ok(LalafellBot {
      discord: client,
      env,
    })
  }
}

pub fn is_administrator<U: Into<UserId>>(user: U) -> LalafellResult<bool> {
  use lalafell::error::ResultExt;
  let user_id = user.into();
  let number_matching: i64 = crate::bot::with_connection(|c| {
    use crate::database::schema::administrators::dsl;
    dsl::administrators
      .find(user_id.to_u64())
      .count()
      .get_result(c)
  }).chain_err(|| "could not check administrators")?;
  Ok(number_matching > 0)
}
