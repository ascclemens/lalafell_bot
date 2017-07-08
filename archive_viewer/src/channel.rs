use MESSAGES;

use iron::prelude::*;
use iron::status;

use router::Router;

use handlebars_iron::Template;

use discord::model::{Message, Role, Member, Emoji, ChannelId};

pub fn channel(req: &mut Request) -> IronResult<Response> {
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

  // TODO: embeds
  // TODO: reactions

  response.set_mut(Template::new("channel", data)).set_mut(status::Ok);

  Ok(response)
}

#[derive(Debug, Serialize)]
pub struct ArchiveData<'a> {
  pub channel_name: &'a str,
  pub topic: String,
  pub messages: Vec<(&'a Message, MessageData)>,
  pub channel_id: u64,
  pub prev_page: u64,
  pub current_page: u64,
  pub next_page: u64,
  pub pages: u64,
  pub start: bool,
  pub end: bool
}

#[derive(Debug, Serialize)]
pub struct MessageData {
  pub timestamp: String,
  pub name_color: String
}

#[derive(Debug, Deserialize)]
pub struct Archive {
  pub server: ArchiveServer,
  pub channel: ArchiveChannel,
  pub messages: Vec<Message>
}

#[derive(Debug, Deserialize)]
pub struct ArchiveServer {
  pub name: String,
  pub roles: Vec<Role>,
  pub members: Vec<Member>,
  pub channels: Vec<ArchiveChannel>,
  pub icon: Option<String>,
  pub emojis: Vec<Emoji>
}

#[derive(Debug, Deserialize)]
pub struct ArchiveChannel {
  pub id: ChannelId,
  pub name: String,
  pub topic: Option<String>
}
