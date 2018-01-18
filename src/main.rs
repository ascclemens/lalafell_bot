#![feature(mpsc_select, box_syntax, fnbox)]
// areyoufuckingkiddingme.jpg
#![cfg_attr(feature = "cargo-clippy", allow(unreadable_literal))]
#![recursion_limit = "1024"]

extern crate ansi_term;
extern crate byteorder;
extern crate chrono;
extern crate ctrlc;
#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate envy;
#[macro_use]
extern crate error_chain;
extern crate fern;
extern crate hyper_rustls;
extern crate hyper;
extern crate itertools;
extern crate lalafell;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate make_hyper_great_again;
extern crate rand;
extern crate scraper;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serenity;
extern crate typemap;
extern crate url_serde;
extern crate url;
extern crate uuid;
extern crate xivdb;

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
  pub discord_bot_token: String,
  pub database_location: String,
  pub config_location: String
}
