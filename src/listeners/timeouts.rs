use bot::LalafellBot;
use listeners::ReceivesEvents;
use discord::model::{Event, Message, Channel, PublicChannel};

use chrono::prelude::*;

use std::sync::Arc;

#[allow(dead_code)]
pub struct Timeouts {
  bot: Arc<LalafellBot>
}

impl Timeouts {
  pub fn new(bot: Arc<LalafellBot>) -> Self {
    Timeouts {
      bot: bot
    }
  }

  fn handle_message(&self, message: &Message) {
    let channel = match self.bot.discord.get_channel(message.channel_id) {
      Ok(Channel::Public(c)) => c,
      _ => return
    };
    let ends = {
      let database = self.bot.database.read().unwrap();
      match database.timeouts.iter().find(|t| t.user_id == message.author.id.0 && t.server_id == channel.server_id.0) {
        Some(t) => t.ends(),
        None => return
      }
    };

    if ends < Utc::now().timestamp() {
      let mut database = self.bot.database.write().unwrap();
      let index = match database.timeouts.iter().position(|t| t.user_id == message.author.id.0 && t.server_id == channel.server_id.0) {
        Some(i) => i,
        None => return
      };
      database.timeouts.remove(index);
      return;
    }

    if let Err(e) = self.bot.discord.delete_message(message.channel_id, message.id) {
      warn!("could not delete message {} in {}: {}", message.id.0, message.channel_id.0, e);
    }
  }

  fn handle_channel_create(&self, channel: &PublicChannel) {
    let state_option = self.bot.state.read().unwrap();
    let state = state_option.as_ref().unwrap();
    let server = match state.servers().iter().find(|s| s.id == channel.server_id) {
      Some(s) => s,
      None => return
    };
    if let Err(e) = ::commands::timeout::set_up_timeouts(self.bot.as_ref(), server) {
      warn!("could not add timeout overwrite to {}: {}", channel.id.0, e);
    }
  }
}

impl ReceivesEvents for Timeouts {
  fn receive(&self, event: &Event) {
    match *event {
      Event::MessageCreate(ref m) => self.handle_message(m),
      Event::ChannelCreate(Channel::Public(ref c)) => self.handle_channel_create(c),
      _ => {}
    }
  }
}
