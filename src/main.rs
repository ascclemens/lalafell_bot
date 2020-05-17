#![feature(box_syntax, box_patterns, never_type, maybe_uninit_ref, maybe_uninit_extra)]
// areyoufuckingkiddingme.jpg
#![allow(proc_macro_derive_resolution_fallback, clippy::unreadable_literal)]
#![recursion_limit = "1024"]

#[macro_use]
extern crate bot_command_derive;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate error_chain;
extern crate ffxiv_types as ffxiv;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate structopt;

// TODO: Efficiency. Every time a command is called, it creates a new App and calls the methods on
//       it. Storing just one App per command would be ideal.

macro_rules! into {
  ($t:ty, $e:expr) => {{
    let x: $t = $e.into();
    x
  }}
}

macro_rules! some_or {
  ($e: expr, $o: expr) => {{
    #[allow(unused_variables)]
    match $e {
      Some(x) => x,
      None => $o
    }
  }}
}

mod bot;
mod commands;
mod config;
mod database;
mod error;
mod filters;
mod listeners;
mod lodestone;
mod logging;
mod tasks;
mod util;

use crate::{
  bot::LalafellBot,
  error::*,
};

use std::sync::{Arc, Mutex};

fn main() {
  if let Err(e) = inner() {
    for err in e.iter() {
      error!("{}", err);
    }
  }
}

fn inner() -> Result<()> {
  if let Err(e) = logging::init_logger() {
    eprintln!("Could not set up logger.");
    for err in e.iter() {
      eprintln!("{}", err);
    }
    return Ok(());
  }

  info!("Loading .env");

  dotenv::dotenv().ok();

  info!("Reading environment variables");

  let environment: Environment = envy::prefixed("LB_").from_env().expect("Invalid or missing environment variables");

  let bot = match bot::create_bot(environment) {
    Ok(b) => b,
    Err(e) => bail!("could not create bot: {}", e),
  };

  let shard_manager = Arc::clone(&bot.discord.shard_manager);

  ctrlc::set_handler(move || {
    info!("Stopping main loop");
    shard_manager.lock().shutdown_all();
  }).expect("could not set interrupt handler");

  let bot = Arc::new(Mutex::new(bot));

  info!("Spinning up bot");
  std::thread::spawn(move || {
    let mut bot = bot.lock().unwrap();
    if let Err(e) = bot.discord.start_autosharded() {
      error!("could not start bot: {}", e);
    }
  }).join().unwrap();

  info!("Exiting");
  Ok(())
}

#[derive(Debug, Deserialize)]
pub struct Environment {
  pub config: String,
  pub database_location: String,
  pub discord_bot_token: String,
  pub fflogs_api_key: String,
}
