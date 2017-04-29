use LalafellBot;
use commands::*;
use commands::tag::Tagger;

use discord::builders::EmbedBuilder;
use discord::model::{Channel, UserId, Role};
use discord::model::permissions;

use xivdb::error::*;

use std::sync::Arc;

const USAGE: &'static str = "!tag <who> <server> <character>";

pub struct TagCommand {
  bot: Arc<LalafellBot>
}

impl TagCommand {
  pub fn new(bot: Arc<LalafellBot>) -> TagCommand {
    TagCommand {
      bot: bot
    }
  }
}

impl<'a> Command<'a> for TagCommand {
  fn run(&self, message: &Message, params: &[&str]) -> CommandResult<'a> {
    let channel = self.bot.discord.get_channel(message.channel_id).chain_err(|| "could not get channel for message")?;
    let server_id = match channel {
      Channel::Public(c) => c.server_id,
      _ => {
        let err: error::Error = "channel was not public".into();
        return Err(err.into());
      }
    };
    let user = self.bot.discord.get_member(server_id, message.author.id).chain_err(|| "could not get member for message")?;
    let mut state_option = self.bot.state.lock().unwrap();
    let state = state_option.as_mut().unwrap();
    let server = match state.servers().iter().find(|x| x.id == server_id) {
      Some(s) => s,
      None => {
        let err: error::Error = "could not find server for channel".into();
        return Err(err.into());
      }
    };
    if server.owner_id != message.author.id {
      let roles = &server.roles;
      let user_roles: Option<Vec<&Role>> = user.roles.iter()
        .map(|r| roles.iter().find(|z| z.id == *r))
        .collect();
      let can_manage_roles = match user_roles {
        Some(ur) => ur.iter().any(|r| r.permissions.contains(permissions::MANAGE_ROLES)),
        None => false
      };
      if !can_manage_roles {
        return Err(ExternalCommandFailure::default()
          .message(|e: EmbedBuilder| e
            .title("Not enough permissions.")
            .description("You don't have enough permissions to use this command."))
          .wrap());
      }
    }

    if params.len() < 4 {
      return Err(ExternalCommandFailure::default()
        .message(|e: EmbedBuilder| e
          .title("Not enough parameters.")
          .description(USAGE))
        .wrap());
    }

    let who = params[0];
    let who = if !who.starts_with("<@") && !who.ends_with('>') && message.mentions.len() != 1 {
      who.parse::<u64>().map(UserId).map_err(|_| ExternalCommandFailure::default()
        .message(|e: EmbedBuilder| e
          .title("Invalid target.")
          .description("The target was not a mention, and it was not a user ID."))
        .wrap())?
    } else {
      message.mentions[0].id
    };
    let ff_server = params[1];
    let name = params[2..].join(" ");

    match Tagger::search_tag(self.bot.clone(), who, server, ff_server, &name)? {
      Some(error) => Err(ExternalCommandFailure::default()
        .message(move |e: EmbedBuilder| e.description(&error))
        .wrap()),
      None => Ok(CommandSuccess::default())
    }
  }
}
