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

mod error;
use error::*;

use iron::prelude::*;
use iron::status;

use router::Router;

use mount::Mount;

use staticfile::Static;

use handlebars_iron::{HandlebarsEngine, DirectorySource, Template};
use handlebars_iron::handlebars::*;

use discord::model::Message;

use std::path::Path;
use std::fs::File;

lazy_static! {
  static ref MESSAGES: Vec<Message> = get_messages().unwrap();
}

fn handlebars() -> Result<HandlebarsEngine> {
  fn range(h: &Helper, r: &Handlebars, rc: &mut RenderContext) -> std::result::Result<(), RenderError> {
    let param = h.params()[0].value().as_u64().unwrap();
    for i in 0..param {
      rc.push_block_context(&(i + 1));
      h.template().map(|t| t.render(r, rc)).unwrap_or(Ok(())).unwrap();
    }
    Ok(())
  }

  fn eq(h: &Helper, r: &Handlebars, rc: &mut RenderContext) -> std::result::Result<(), RenderError> {
    let param_1 = h.params()[0].value();
    let param_2 = h.params()[1].value();
    if param_1 == param_2 {
      h.template().map(|t| t.render(r, rc)).unwrap_or(Ok(())).unwrap();
    }
    Ok(())
  }

  let mut handlebars = Handlebars::new();
  handlebars.register_helper("range", box range);
  handlebars.register_helper("eq", box eq);

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
    index: get "/:channel_id/:page" => channel
  )
}

fn chain(mount: Mount, handlebars: HandlebarsEngine) -> Chain {
  let mut chain = Chain::new(mount);
  chain.link_after(handlebars);
  chain
}

fn main() {
  let handlebars = handlebars().unwrap();
  let router = router();
  let mount = mount(router);
  let chain = chain(mount, handlebars);

  Iron::new(chain).http("localhost:3000").unwrap();
}

fn channel(req: &mut Request) -> IronResult<Response> {
  let params = req.extensions.get::<Router>().unwrap();
  let channel_id = match params.find("channel_id") {
    Some(c) => c,
    None => return Ok(Response::with(("No channel_id", status::BadRequest)))
  };
  let channel_id = match channel_id.parse::<u64>() {
    Ok(c) => c,
    Err(_) => return Ok(Response::with(("Bad channel_id", status::BadRequest)))
  };
  let page = params.find("page").unwrap_or("1");
  let page = match page.parse::<u64>() {
    Ok(p) if p > 0 => p - 1,
    _ => return Ok(Response::with(("Bad page number", status::BadRequest)))
  };

  let mut response = Response::new();

  let message_chunk = match MESSAGES.chunks(50).nth(page as usize) {
    Some(m) => m,
    None => return Ok(Response::with(("No such page", status::BadRequest)))
  };
  let wrappers: Vec<_> = message_chunk.into_iter()
    .map(|m| {
      let timestamp = m.timestamp.format("%m/%d/%Y at %H:%M:%S").to_string();
      (m, timestamp)
    })
    .rev()
    .collect();
  let pages = (MESSAGES.len() as f32 / 50.0).ceil() as u64;
  let data = ArchiveData {
    messages: wrappers,
    channel_id: channel_id,
    prev_page: page,
    current_page: page + 1,
    next_page: page + 2,
    pages: pages,
    start: page == 0,
    end: page == pages - 1
  };

  // TODO: page links

  response.set_mut(Template::new("index", data)).set_mut(status::Ok);

  Ok(response)
}

fn get_messages() -> Result<Vec<Message>> {
  let f = File::open("archives/322773022064771073.json").chain_err(|| "could not open 322773022064771073.json")?;
  serde_json::from_reader(f).chain_err(|| "could not parse 322773022064771073.json")
}

#[derive(Debug, Serialize)]
struct ArchiveData<'a> {
  messages: Vec<(&'a Message, String)>,
  channel_id: u64,
  prev_page: u64,
  current_page: u64,
  next_page: u64,
  pages: u64,
  start: bool,
  end: bool
}
