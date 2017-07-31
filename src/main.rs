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
extern crate make_hyper_great_again;
extern crate hyper_rustls;
extern crate ansi_term;
extern crate scraper;
extern crate uuid;
#[macro_use]
extern crate lazy_static;
extern crate envy;
#[macro_use]
extern crate error_chain;
extern crate lalafell;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_codegen;
extern crate url;
extern crate url_serde;

mod bot;
mod database;
mod listeners;
mod tasks;
mod commands;
mod lodestone;
mod config;
mod error;

use bot::LalafellBot;
use error::*;

use std::sync::mpsc::channel;

// TODO: move delete_all_messages to database
// TODO: move tag_instructions to database
// TODO: change tag_instructions name

fn main() {
  if let Err(e) = inner() {
    for err in e.iter() {
      error!("{}", err);
    }
  }
}

fn inner() -> Result<()> {
  if let Err(e) = lalafell::logging::init_logger() {
    println!("Could not set up logger.");
    for err in e.iter() {
      println!("{}", err);
    }
    return Ok(());
  }

  info!("Loading .env");

  dotenv::dotenv().ok();

  info!("Reading environment variables");

  let environment: Environment = envy::from_env().expect("Invalid or missing environment variables");

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
