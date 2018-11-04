#![feature(mpsc_select, box_syntax, fnbox, never_type)]
// areyoufuckingkiddingme.jpg
#![allow(proc_macro_derive_resolution_fallback, clippy::unreadable_literal)]
#![recursion_limit = "1024"]

extern crate ansi_term;
#[macro_use]
extern crate bot_command_derive;
extern crate byteorder;
extern crate chrono;
extern crate ctrlc;
#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate envy;
#[macro_use]
extern crate error_chain;
extern crate failure;
extern crate fern;
extern crate fflogs;
extern crate ffxiv_types as ffxiv;
extern crate itertools;
extern crate lalafell;
#[macro_use]
extern crate lazy_static;
extern crate lodestone_api_client;
#[macro_use]
extern crate log;
extern crate rand;
extern crate reqwest;
extern crate scraper;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serenity;
#[macro_use]
extern crate structopt;
extern crate typemap;
extern crate unicase;
extern crate url;
extern crate uuid;

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

use error::*;
use bot::LalafellBot;

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
    println!("Could not set up logger.");
    for err in e.iter() {
      println!("{}", err);
    }
    return Ok(());
  }

  info!("Loading .env");

  dotenv::dotenv().ok();

  info!("Reading environment variables");

  let environment: Environment = envy::prefixed("LB_").from_env().expect("Invalid or missing environment variables");

  let bot = match bot::create_bot(environment) {
    Ok(b) => b,
    Err(e) => bail!("could not create bot: {}", e)
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
