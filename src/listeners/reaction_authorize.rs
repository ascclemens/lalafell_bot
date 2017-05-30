use LalafellBot;
use config::Listener;
use listeners::ReceivesEvents;
use discord::model::{Event, Channel, ReactionEmoji};

use xivdb::error::*;

use serde_json;

use std::sync::Arc;

pub struct ReactionAuthorize {
  bot: Arc<LalafellBot>,
  config: ReactionAuthorizeConfig
}

impl ReactionAuthorize {
  pub fn new(bot: Arc<LalafellBot>, listener: &Listener) -> Result<ReactionAuthorize> {
    let config = match listener.config {
      Some(ref c) => serde_json::from_value(c.clone()).chain_err(|| "could not parse reaction_authorize configuration")?,
      None => return Err("missing configuration for reaction_authorize listener".into())
    };
    Ok(ReactionAuthorize {
      bot: bot,
      config: config
    })
  }
}

impl ReceivesEvents for ReactionAuthorize {
  fn receive(&self, event: &Event) {
    let (added, reaction) = match *event {
      Event::ReactionAdd(ref reaction) => (true, reaction),
      Event::ReactionRemove(ref reaction) => (false, reaction),
      _ => return
    };
    if reaction.message_id.0 != self.config.message {
      return;
    }
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
    match reaction.emoji {
      ReactionEmoji::Unicode(ref emoji) if *emoji == self.config.emoji => {},
      _ => return
    }
    let roles = match self.bot.discord.get_roles(channel.server_id) {
      Ok(r) => r,
      Err(e) => {
        warn!("couldn't get roles: {}", e);
        return;
      }
    };
    let role = match roles.iter().find(|r| r.name == self.config.role) {
      Some(r) => r,
      None => {
        warn!("couldn't find role for name \"{}\"", self.config.role);
        return;
      }
    };
    let result = if added {
      self.bot.discord.add_user_to_role(channel.server_id, reaction.user_id, role.id)
    } else {
      self.bot.discord.remove_user_from_role(channel.server_id, reaction.user_id, role.id)
    };
    if let Err(e) = result {
      warn!("could not change role: {}", e);
    }
  }
}

#[derive(Debug, Deserialize)]
pub struct ReactionAuthorizeConfig {
  message: u64,
  emoji: String,
  role: String
}
