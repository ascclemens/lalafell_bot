use {LalafellBot, Environment};
use config::Config;

use xivdb::error::*;

use serde_json;

use std::fs::File;
use std::sync::Arc;

mod listeners;
mod tasks;

use self::listeners::listeners;
use self::tasks::tasks;

pub fn create_bot(environment: Environment) -> Result<Arc<LalafellBot>> {
  info!("Loading configuration");
  let config = config(&environment)?;
  info!("Constructing bot");
  let bot = Arc::new(LalafellBot::new(environment, config).chain_err(|| "could not create bot")?);
  info!("Registering listeners");
  listeners(bot.clone())?;
  info!("Starting tasks");
  tasks(bot.clone())?;
  Ok(bot)
}

fn config(environment: &Environment) -> Result<Config> {
  match File::open(&environment.config_location) {
    Ok(f) => serde_json::from_reader(f).chain_err(|| "could not parse config"),
    Err(_) => Ok(Default::default())
  }
}
