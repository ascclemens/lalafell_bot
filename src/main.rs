#![feature(mpsc_select, box_syntax, fnbox)]

extern crate discord;
extern crate xivdb;
extern crate dotenv;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate ctrlc;
extern crate chrono;
#[macro_use]
extern crate log;
extern crate fern;
extern crate hyper;
extern crate ansi_term;
extern crate scraper;
extern crate uuid;
#[macro_use]
extern crate lazy_static;

// FIXME: Use envy when it upgrades to serde 1.0

mod logging;
mod bot;
mod database;
mod listeners;
mod tasks;
mod commands;
mod lodestone;
mod config;

use bot::LalafellBot;

use xivdb::error::*;

use std::env::var;
use std::sync::mpsc::channel;

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

  let environment = Environment {
    discord_bot_token: var("LB_DISCORD_BOT_TOKEN").chain_err(|| "No bot token was specified in .env")?,
    database_location: var("LB_DATABASE_LOCATION").chain_err(|| "No database location was specified in .env")?,
    config_location: var("LB_CONFIG_LOCATION").chain_err(|| "No config location was specified in .env")?
  };

  info!("Creating bot");

  let bot = bot::create_bot(environment)?;

  let (loop_cancel_tx, loop_cancel_rx) = channel();

  ctrlc::set_handler(move || {
    info!("Stopping main loop");
    loop_cancel_tx.send(()).unwrap();
  }).expect("could not set interrupt handler");

  info!("Spinning up bot");
  let thread_bot = bot.clone();
  std::thread::spawn(move || {
    if let Err(e) = thread_bot.start_loop(loop_cancel_rx) {
      error!("could not start bot loop: {}", e);
    }
  }).join().unwrap();
  info!("Saving database");
  bot.save_database(None)?;
  info!("Exiting");
  Ok(())
}

#[derive(Debug, Deserialize)]
pub struct Environment {
  #[serde(rename = "lb_discord_bot_token")]
  pub discord_bot_token: String,
  #[serde(rename = "lb_database_location")]
  pub database_location: String,
  #[serde(rename = "lb_config_location")]
  pub config_location: String
}
