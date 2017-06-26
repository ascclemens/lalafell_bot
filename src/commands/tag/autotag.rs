use bot::LalafellBot;
use commands::*;
use commands::tag::Tagger;

use discord::builders::EmbedBuilder;
use discord::model::{LiveServer, PublicChannel};

use std::sync::Arc;

const USAGE: &'static str = "!autotag <server> <character>";

pub struct AutoTagCommand {
  bot: Arc<LalafellBot>
}

impl AutoTagCommand {
  pub fn new(bot: Arc<LalafellBot>) -> AutoTagCommand {
    AutoTagCommand {
      bot: bot
    }
  }
}

impl HasBot for AutoTagCommand {
  fn bot(&self) -> Arc<LalafellBot> {
    self.bot.clone()
  }
}

impl<'a> PublicChannelCommand<'a> for AutoTagCommand {
  fn run(&self, message: &Message, server: &LiveServer, _: &PublicChannel, params: &[&str]) -> CommandResult<'a> {
    if params.len() < 3 {
      return Err(ExternalCommandFailure::default()
        .message(|e: EmbedBuilder| e
          .title("Not enough parameters.")
          .description(USAGE))
        .wrap());
    }

    let ff_server = params[0];
    let name = params[1..].join(" ");

    match Tagger::search_tag(self.bot.as_ref(), message.author.id, server, ff_server, &name, false)? {
      Some(error) => Err(ExternalCommandFailure::default()
        .message(move |e: EmbedBuilder| e.description(&error))
        .wrap()),
      None => Ok(CommandSuccess::default())
    }
  }
}
