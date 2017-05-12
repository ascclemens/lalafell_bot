use LalafellBot;
use commands::*;
use commands::tag::Tagger;

use discord::builders::EmbedBuilder;
use discord::model::{PublicChannel, UserId, Role};
use discord::model::permissions;

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

impl HasBot for TagCommand {
  fn bot(&self) -> Arc<LalafellBot> {
    self.bot.clone()
  }
}

impl<'a> PublicChannelCommand<'a> for TagCommand {
  fn run(&self, message: &Message, channel: &PublicChannel, params: &[&str]) -> CommandResult<'a> {
    let server_id = channel.server_id;
    let user = self.bot.discord.get_member(server_id, message.author.id).chain_err(|| "could not get member for message")?;
    let state_option = self.bot.state.read().unwrap();
    let state = state_option.as_ref().unwrap();
    let server = match state.servers().iter().find(|x| x.id == server_id) {
      Some(s) => s,
      None => {
        let err: error::Error = "could not find server for channel".into();
        return Err(err.into());
      }
    };
    let can_manage_roles = if server.owner_id != message.author.id {
      true
    } else {
      let roles = &server.roles;
      let user_roles: Option<Vec<&Role>> = user.roles.iter()
        .map(|r| roles.iter().find(|z| z.id == *r))
        .collect();
      match user_roles {
        Some(ur) => ur.iter().any(|r| r.permissions.contains(permissions::MANAGE_ROLES)),
        None => false
      }
    };
    if !can_manage_roles {
      return Err(ExternalCommandFailure::default()
        .message(|e: EmbedBuilder| e
          .title("Not enough permissions.")
          .description("You don't have enough permissions to use this command."))
        .wrap());
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

    match Tagger::search_tag(self.bot.clone(), who, server, ff_server, &name, can_manage_roles)? {
      Some(error) => Err(ExternalCommandFailure::default()
        .message(move |e: EmbedBuilder| e.description(&error))
        .wrap()),
      None => Ok(CommandSuccess::default())
    }
  }
}
