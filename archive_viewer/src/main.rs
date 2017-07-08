#![feature(box_syntax)]

extern crate iron;
#[macro_use]
extern crate router;
extern crate mount;
extern crate staticfile;
extern crate handlebars_iron;
#[macro_use]
extern crate error_chain;
extern crate discord;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate lazy_static;
extern crate url;
extern crate chrono;

mod error;

mod index;
mod channel;
mod refresh;
mod helpers;

use error::*;
use channel::*;
use refresh::*;

use iron::prelude::*;

use router::Router;

use mount::Mount;

use staticfile::Static;

use handlebars_iron::{HandlebarsEngine, DirectorySource};
use handlebars_iron::handlebars::*;

use std::path::Path;
use std::sync::RwLock;
use std::collections::HashMap;

lazy_static! {
  pub static ref MESSAGES: RwLock<HashMap<u64, HashMap<u64, Archive>>> = RwLock::default();
}

fn handlebars() -> Result<HandlebarsEngine> {
  let mut handlebars = Handlebars::new();
  helpers::add_helpers(&mut handlebars);

  let mut engine = HandlebarsEngine::from(handlebars);
  engine.add(box DirectorySource::new("web/templates", ".hbs"));

  engine.reload().chain_err(|| "error loading handlebars")?;

  Ok(engine)
}

fn mount(router: Router) -> Mount {
  let mut mount = Mount::new();
  mount.mount("/static", Static::new(Path::new("web/static")));
  mount.mount("/", router);
  mount
}

fn router() -> Router {
  router!(
    index: get "/" => index::index,
    channel_no_page: get "/:server_id/:channel_id" => channel::channel_redirect,
    channel: get "/:server_id/:channel_id/:page" => channel::channel,
    refresh: get "/refresh" => refresh::refresh
  )
}

fn chain(mount: Mount, handlebars: HandlebarsEngine) -> Chain {
  let mut chain = Chain::new(mount);
  chain.link_after(handlebars);
  chain
}

fn main() {
  println!("Starting up");

  println!("Creating Handlebars instance");
  let handlebars = handlebars().unwrap();

  println!("Creating router");
  let router = router();

  println!("Creating mount");
  let mount = mount(router);

  println!("Creating chain");
  let chain = chain(mount, handlebars);

  println!("Refreshing database");
  _refresh();

  println!("Starting server on localhost:3000");
  Iron::new(chain).http("localhost:3000").unwrap();
}
