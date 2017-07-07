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

use discord::model::{Message, Role, Member, Emoji, ChannelId};

use std::path::{Path, PathBuf};
use std::fs::File;
use std::sync::RwLock;
use std::collections::HashMap;

lazy_static! {
  static ref MESSAGES: RwLock<HashMap<u64, HashMap<u64, Archive>>> = RwLock::default();
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
    channel: get "/:server_id/:channel_id/:page" => channel,
    refresh: get "/refresh" => refresh
  )
}

fn add_messages(channel: PathBuf, server_id: u64, channel_id: u64) {
  let f = File::open(channel).unwrap();
  let mut archive: Archive = serde_json::from_reader(f).unwrap();
  for message in &mut archive.messages {
    if let Some(member) = archive.server.members.iter().find(|mem| mem.user.id == message.author.id) {
      if let Some(ref nick) = member.nick {
        message.author.name = nick.clone();
      }
    }
    let mut parts: Vec<String> = message.content.split(' ').map(ToOwned::to_owned).collect();
    for part in &mut parts {
      if part.starts_with("<@!") {
        let end = part.find('>').unwrap_or_else(|| part.len() - 1);
        let id: u64 = match part[3..end].parse() {
          Ok(u) => u,
          Err(_) => continue
        };
        if let Some(member) = archive.server.members.iter().find(|m| m.user.id.0 == id) {
          let name = member.nick.as_ref().unwrap_or(&member.user.name);
          *part = format!("@{}", name);
        }
      } else if part.starts_with("<@&") {
        let end = part.find('>').unwrap_or_else(|| part.len() - 1);
        let id: u64 = match part[3..end].parse() {
          Ok(u) => u,
          Err(_) => continue
        };
        if let Some(role) = archive.server.roles.iter().find(|r| r.id.0 == id) {
          let name = if role.name == "@everyone" { role.name.clone() } else { format!("@{}", role.name) };
          *part = name;
        }
      } else if part.starts_with("<@") {
        let end = part.find('>').unwrap_or_else(|| part.len() - 1);
        let id: u64 = match part[2..end].parse() {
          Ok(u) => u,
          Err(_) => continue
        };
        if let Some(member) = archive.server.members.iter().find(|m| m.user.id.0 == id) {
          let name = member.nick.as_ref().unwrap_or(&member.user.name);
          *part = format!("@{}", name);
        }
      } else if part.starts_with("<#") {
        let end = part.find('>').unwrap_or_else(|| part.len() - 1);
        let id: u64 = match part[2..end].parse() {
          Ok(u) => u,
          Err(_) => continue
        };
        if let Some(channel) = archive.server.channels.iter().find(|c| c.id.0 == id) {
          *part = format!("#{}", channel.name);
        }
      }
    }
    message.content = parts.join(" ");
  }
  let mut msgs = MESSAGES.write().unwrap();
  let server = msgs.entry(server_id).or_insert_with(Default::default);
  server.insert(channel_id, archive);
}

fn _refresh() {
  let archives = std::fs::read_dir("archives").unwrap();
  for server in archives {
    let server = server.unwrap().path();
    if server.is_dir() {
      let server_id: u64 = server.file_stem().unwrap().to_string_lossy().parse().unwrap();
      for channel in std::fs::read_dir(server).unwrap() {
        let channel = channel.unwrap().path();
        let channel_id: u64 = channel.file_stem().unwrap().to_string_lossy().parse().unwrap();
        add_messages(channel, server_id, channel_id);
      }
    }
  }
}

fn refresh(_: &mut Request) -> IronResult<Response> {
  _refresh();
  Ok(Response::with(("We gucci", status::Ok)))
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

  _refresh();

  Iron::new(chain).http("localhost:3000").unwrap();
}

fn channel(req: &mut Request) -> IronResult<Response> {
  let params = req.extensions.get::<Router>().unwrap();
  let server_id = match params.find("server_id") {
    Some(c) => c,
    None => return Ok(Response::with(("No server_id", status::BadRequest)))
  };
  let server_id = match server_id.parse::<u64>() {
    Ok(c) => c,
    Err(_) => return Ok(Response::with(("Bad server_id", status::BadRequest)))
  };
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

  let msgs = MESSAGES.read().unwrap();
  let server = match msgs.get(&server_id) {
    Some(s) => s,
    None => return Ok(Response::with(("No such server", status::BadRequest)))
  };
  let archive = match server.get(&channel_id) {
    Some(c) => c,
    None => return Ok(Response::with(("No such channel", status::BadRequest)))
  };
  let messages = &archive.messages;

  let message_chunk = match messages.chunks(50).nth(page as usize) {
    Some(m) => m,
    None => return Ok(Response::with(("No such page", status::BadRequest)))
  };
  let wrappers: Vec<_> = message_chunk.into_iter()
    .map(|m| {
      let timestamp = m.timestamp.format("%m/%d/%Y at %H:%M:%S").to_string();
      let mut color = 0;
      if let Some(member) = archive.server.members.iter().find(|mem| mem.user.id == m.author.id) {
        let roles = member.roles.iter().flat_map(|r| archive.server.roles.iter().find(|x| x.id == *r));
        if let Some(role) = roles.max_by_key(|r| r.position) {
          color = role.color;
        }
      }
      let data = MessageData {
        timestamp: timestamp,
        name_color: format!("#{:x}", color)
      };
      (m, data)
    })
    .rev()
    .collect();
  let pages = (messages.len() as f32 / 50.0).ceil() as u64;
  let data = ArchiveData {
    channel_name: &archive.channel.name,
    topic: archive.channel.topic.clone().unwrap_or_else(Default::default),
    messages: wrappers,
    channel_id: channel_id,
    prev_page: page,
    current_page: page + 1,
    next_page: page + 2,
    pages: pages,
    start: page == 0,
    end: page == pages - 1
  };

  // TODO: special color for roles, mentions, and channels
  // TODO: embeds
  // TODO: links

  response.set_mut(Template::new("channel", data)).set_mut(status::Ok);

  Ok(response)
}

#[derive(Debug, Serialize)]
struct ArchiveData<'a> {
  channel_name: &'a str,
  topic: String,
  messages: Vec<(&'a Message, MessageData)>,
  channel_id: u64,
  prev_page: u64,
  current_page: u64,
  next_page: u64,
  pages: u64,
  start: bool,
  end: bool
}

#[derive(Debug, Serialize)]
struct MessageData {
  timestamp: String,
  name_color: String
}

#[derive(Debug, Deserialize)]
struct Archive {
  server: ArchiveServer,
  channel: ArchiveChannel,
  messages: Vec<Message>
}

#[derive(Debug, Deserialize)]
struct ArchiveServer {
  name: String,
  roles: Vec<Role>,
  members: Vec<Member>,
  channels: Vec<ArchiveChannel>,
  icon: Option<String>,
  emojis: Vec<Emoji>
}

#[derive(Debug, Deserialize)]
struct ArchiveChannel {
  id: ChannelId,
  name: String,
  topic: Option<String>
}
