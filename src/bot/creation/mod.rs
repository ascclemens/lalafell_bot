use {LalafellBot, Environment};
use config::Config;

use error::*;

use serde_json;

use std::fs::File;

mod listeners;
mod tasks;

use self::tasks::tasks;
use bot::data::data;

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

fn config(environment: &Environment) -> Result<Config> {
  match File::open(&environment.config_location) {
    Ok(f) => serde_json::from_reader(f).chain_err(|| "could not parse config"),
    Err(_) => Ok(Default::default())
  }
}
