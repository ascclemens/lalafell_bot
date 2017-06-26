use bot::LalafellBot;
use commands::*;
use commands::tag::Tagger;

use discord::builders::EmbedBuilder;
use discord::model::{LiveServer, PublicChannel, UserId};
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

#[derive(Debug, Deserialize)]
pub struct Params {
  who: String,
  server: String,
  name: (String, String)
}

impl HasParams for TagCommand {
  type Params = Params;
}

impl<'a> PublicChannelCommand<'a> for TagCommand {
  fn run(&self, message: &Message, server: &LiveServer, channel: &PublicChannel, params: &[&str]) -> CommandResult<'a> {
    let params = self.params(USAGE, params)?;
    let can_manage_roles = server.permissions_for(channel.id, message.author.id).contains(permissions::MANAGE_ROLES);
    if !can_manage_roles {
      return Err(ExternalCommandFailure::default()
        .message(|e: EmbedBuilder| e
          .title("Not enough permissions.")
          .description("You don't have enough permissions to use this command."))
        .wrap());
    }

    let who = params.who;
    let who = if !who.starts_with("<@") && !who.ends_with('>') && message.mentions.len() != 1 {
      who.parse::<u64>().map(UserId).map_err(|_| ExternalCommandFailure::default()
        .message(|e: EmbedBuilder| e
          .title("Invalid target.")
          .description("The target was not a mention, and it was not a user ID."))
        .wrap())?
    } else {
      message.mentions[0].id
    };
    let ff_server = params.server;
    let name = params.name.0 + " " + &params.name.1;

    match Tagger::search_tag(self.bot.as_ref(), who, server, &ff_server, &name, can_manage_roles)? {
      Some(error) => Err(ExternalCommandFailure::default()
        .message(move |e: EmbedBuilder| e.description(&error))
        .wrap()),
      None => Ok(CommandSuccess::default())
    }
  }
}
