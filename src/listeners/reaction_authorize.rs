use bot::LalafellBot;
use listeners::ReceivesEvents;
use database::models::Reaction;

use diesel::prelude::*;

use discord::model::{Event, Channel, ReactionEmoji};

use error::*;

use std::sync::Arc;

pub struct ReactionAuthorize {
  bot: Arc<LalafellBot>
}

impl ReactionAuthorize {
  pub fn new(bot: Arc<LalafellBot>) -> ReactionAuthorize {
    ReactionAuthorize {
      bot
    }
  }
}

impl ReceivesEvents for ReactionAuthorize {
  fn receive(&self, event: &Event) {
    let (added, reaction) = match *event {
      Event::ReactionAdd(ref reaction) => (true, reaction),
      Event::ReactionRemove(ref reaction) => (false, reaction),
      _ => return
    };
    let channel = match self.bot.discord.get_channel(reaction.channel_id) {
      Ok(Channel::Public(c)) => c,
      Ok(_) => {
        warn!("invalid channel: {}", reaction.channel_id.0);
        return;
      },
      Err(e) => {
        warn!("couldn't get channel: {}", e);
        return;
      }
    };
    let emoji = match reaction.emoji {
      ReactionEmoji::Unicode(ref emoji)  => emoji,
      _ => return
    };
    let reactions: ::std::result::Result<Vec<Reaction>, _> = ::bot::CONNECTION.with(|c| {
      use database::schema::reactions::dsl;
      dsl::reactions
        .filter(dsl::channel_id.eq(reaction.channel_id.0.to_string())
          .and(dsl::server_id.eq(channel.server_id.0.to_string()))
          .and(dsl::message_id.eq(reaction.message_id.0.to_string()))
          .and(dsl::emoji.eq(emoji)))
        .load(c)
        .chain_err(|| "could not load reactions")
    });
    let reactions = match reactions {
      Ok(r) => r,
      Err(e) => {
        warn!("couldn't get reactions: {}", e);
        return;
      }
    };
    let roles = match self.bot.discord.get_roles(channel.server_id) {
      Ok(r) => r,
      Err(e) => {
        warn!("couldn't get roles: {}", e);
        return;
      }
    };
    for reac in reactions {
      let role = match roles.iter().find(|r| r.name == reac.role) {
        Some(r) => r,
        None => {
          warn!("couldn't find role for name \"{}\"", reac.role);
          continue;
        }
      };
      let result = if added {
        self.bot.discord.add_member_role(channel.server_id, reaction.user_id, role.id)
      } else {
        self.bot.discord.remove_member_role(channel.server_id, reaction.user_id, role.id)
      };
      if let Err(e) = result {
        warn!("could not change role: {}", e);
      }
    }
  }
}

#[derive(Debug, Deserialize)]
pub struct ReactionAuthorizeConfig {
  message: u64,
  emoji: String,
  role: String
}
