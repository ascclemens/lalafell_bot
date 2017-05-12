use LalafellBot;
use config::Listener;
use listeners::ReceivesEvents;
use discord::model::{Event, Channel, ChannelId};

use xivdb::error::*;

use serde_json;

use std::sync::Arc;

pub struct TagInstructions {
  bot: Arc<LalafellBot>,
  config: TagInstructionsConfig
}

impl TagInstructions {
  pub fn new(bot: Arc<LalafellBot>, listener: &Listener) -> Result<TagInstructions> {
    let config = match listener.config {
      Some(ref c) => serde_json::from_value(c.clone()).chain_err(|| "could not parse tag_instructions configuration")?,
      None => return Err("missing configuration for tag_instructions listener".into())
    };
    Ok(TagInstructions {
      bot: bot,
      config: config
    })
  }
}

impl ReceivesEvents for TagInstructions {
  fn receive(&self, event: &Event) {
    let destination = ChannelId(self.config.channel);
    let event_data = match *event {
      Event::ServerMemberAdd(ref server_id, ref member) => {
        let channel = match self.bot.discord.get_channel(destination) {
          Ok(Channel::Public(c)) => c,
          _ => return
        };
        if &channel.server_id != server_id {
          return;
        }
        Some(member.clone())
      },
      Event::MessageCreate(ref m) => {
        let chan = match self.bot.discord.get_channel(m.channel_id) {
          Ok(Channel::Public(c)) => c,
          _ => return
        };
        if chan.id != destination {
          return;
        }
        if m.content.to_lowercase().starts_with("!autotag ") {
          return;
        }
        self.bot.discord.get_member(chan.server_id, m.author.id).ok()
      },
      _ => return
    };
    let member = match event_data {
      Some(e) => e,
      None => return
    };

    if !member.roles.is_empty() {
      return;
    }

    let message = self.config.message
      .replace("{mention}", &member.user.mention().to_string());
    self.bot.discord.send_embed(destination, "", |e| e.description(&message)).ok();
  }
}

#[derive(Debug, Deserialize)]
pub struct TagInstructionsConfig {
  channel: u64,
  message: String
}
