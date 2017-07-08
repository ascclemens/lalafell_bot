use MESSAGES;

use channel::Archive;

use iron::prelude::*;
use iron::status;

use handlebars_iron::handlebars::*;

use url::Url;

use serde_json;

use std::path::PathBuf;
use std::fs::{self, File};

pub fn _refresh() {
  let archives = fs::read_dir("archives").unwrap();
  for server in archives {
    let server = server.unwrap().path();
    if server.is_dir() {
      let server_id: u64 = server.file_stem().unwrap().to_string_lossy().parse().unwrap();
      for channel in fs::read_dir(server).unwrap() {
        let channel = channel.unwrap().path();
        let channel_id: u64 = channel.file_stem().unwrap().to_string_lossy().parse().unwrap();
        add_messages(channel, server_id, channel_id);
      }
    }
  }
}

pub fn refresh(_: &mut Request) -> IronResult<Response> {
  _refresh();
  Ok(Response::with(("We gucci", status::Ok)))
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
    // FIXME: continuing here doesn't escape and opens a vulnerability
    // can probably refactor these into individual methods that return, then always escape the end
    // result or base it on a return value
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
          *part = format!("<span class=\"highlight\">@{}</span>", html_escape(name));
        }
      } else if part.starts_with("<@&") {
        let end = part.find('>').unwrap_or_else(|| part.len() - 1);
        let id: u64 = match part[3..end].parse() {
          Ok(u) => u,
          Err(_) => continue
        };
        if let Some(role) = archive.server.roles.iter().find(|r| r.id.0 == id) {
          let name = if role.name == "@everyone" { role.name.clone() } else { format!("@{}", role.name) };
          *part = format!("<span class=\"highlight\">{}</span>", html_escape(&name));
        }
      } else if part.starts_with("<@") {
        let end = part.find('>').unwrap_or_else(|| part.len() - 1);
        let id: u64 = match part[2..end].parse() {
          Ok(u) => u,
          Err(_) => continue
        };
        if let Some(member) = archive.server.members.iter().find(|m| m.user.id.0 == id) {
          let name = member.nick.as_ref().unwrap_or(&member.user.name);
          *part = format!("<span class=\"highlight\">@{}</span>", html_escape(name));
        }
      } else if part.starts_with("<#") {
        let end = part.find('>').unwrap_or_else(|| part.len() - 1);
        let id: u64 = match part[2..end].parse() {
          Ok(u) => u,
          Err(_) => continue
        };
        if let Some(channel) = archive.server.channels.iter().find(|c| c.id.0 == id) {
          *part = format!("<span class=\"highlight\">#{}</span>", html_escape(&channel.name));
        }
      } else if part.starts_with("<:") {
        if let Some(index) = part[2..].find(':') {
          let end = part[2 + index..].find('>').map(|x| x + 2 + index).unwrap_or_else(|| part.len() - 1);
          let id: u64 = match part[3 + index..end].parse() {
            Ok(u) => u,
            Err(_) => continue
          };
          *part = format!("<img class=\"emoji\" alt=\"{}\" src=\"https://cdn.discordapp.com/emojis/{}.png\"/>", &part[2..index], id);
        }
      } else {
        *part = html_escape(part);
      }

      if let Ok(url) = Url::parse(part) {
        if url.has_host() {
          *part = format!("<a href=\"{url}\">{url}</a>", url=part)
        }
      }
    }
    message.content = parts.join(" ");
  }
  let mut msgs = MESSAGES.write().unwrap();
  let server = msgs.entry(server_id).or_insert_with(Default::default);
  server.insert(channel_id, archive);
}
