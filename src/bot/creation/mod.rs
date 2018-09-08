use {LalafellBot, Environment};
use config::Config;

use error::*;

use serde_json;

use std::fs::File;

mod listeners;
mod tasks;
pub mod data;

use self::tasks::tasks;
use self::data::data;

pub use self::listeners::Handler;

pub fn create_bot(environment: Environment) -> Result<LalafellBot> {
  info!("Loading configuration");
  let config = config(&environment)?;
  info!("Constructing bot");
  let bot = LalafellBot::new(environment, config).chain_err(|| "could not create bot")?;
  info!("Starting tasks");
  tasks(&bot)?;
  info!("Registering global data");
  data(&bot);
  Ok(bot)
}

pub fn config(environment: &Environment) -> Result<Config> {
  serde_json::from_str(&environment.config).chain_err(|| "could not parse CONFIG env var")
}
