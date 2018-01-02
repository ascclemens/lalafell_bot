use bot::LalafellBot;
use listeners::ReceivesEvents;
use database::models::Timeout;
use lalafell::error::*;

use discord::model::{Event, Message, Channel, PublicChannel};

use diesel::prelude::*;

use chrono::prelude::*;

use std::sync::Arc;

#[allow(dead_code)]
pub struct Timeouts {
  bot: Arc<LalafellBot>
}

impl Timeouts {
  pub fn new(bot: Arc<LalafellBot>) -> Self {
    Timeouts {
      bot
    }
  }

  fn handle_message(&self, message: &Message) {
    let channel = match self.bot.discord.get_channel(message.channel_id) {
      Ok(Channel::Public(c)) => c,
      _ => return
    };
    let timeout = ::bot::CONNECTION.with(|c| {
      use database::schema::timeouts::dsl;
      dsl::timeouts
        .filter(dsl::user_id.eq(message.author.id.0.to_string()).and(dsl::server_id.eq(channel.server_id.0.to_string())))
        .first(c)
    });

    let timeout: Timeout = match timeout {
      Ok(t) => t,
      _ => return
    };

    if timeout.ends() < Utc::now().timestamp() {
      ::bot::CONNECTION.with(|c| {
        if let Err(e) = ::diesel::delete(&timeout).execute(c).chain_err(|| "could not delete timeout") {
          warn!("could not delete timeout: {}", e);
        }
      });
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
