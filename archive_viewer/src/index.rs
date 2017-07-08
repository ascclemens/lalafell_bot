use MESSAGES;

use iron::prelude::*;
use iron::status;

use handlebars_iron::Template;

use chrono::{Utc, TimeZone};

use std::collections::HashMap;

fn get_servers() -> HashMap<u64, Vec<Channel>> {
  let mut servers = HashMap::new();
  let messages = MESSAGES.read().unwrap();
  for (server_id, channels) in messages.iter() {
    for (channel_id, archive) in channels {
      let channel = Channel {
        name: archive.channel.name.clone(),
        server_name: archive.server.name.clone(),
        server_icon: archive.server.icon.clone(),
        raw_timestamp: archive.timestamp,
        timestamp: Utc.timestamp(archive.timestamp, 0).format("%m/%d/%Y at %H:%M:%S").to_string(),
        id: *channel_id
      };
      servers.entry(*server_id).or_insert_with(Vec::default).push(channel);
    }
  }
  servers
}

pub fn index(_: &mut Request) -> IronResult<Response> {
  let data = Data { servers: get_servers() };
  Ok(Response::with((Template::new("index", data), status::Ok)))
}

#[derive(Debug, Serialize)]
struct Data {
  servers: HashMap<u64, Vec<Channel>>
}

#[derive(Debug, Serialize, Hash)]
struct Channel {
  name: String,
  server_name: String,
  server_icon: Option<String>,
  timestamp: String,
  raw_timestamp: i64,
  id: u64
}
