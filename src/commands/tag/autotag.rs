use bot::LalafellBot;
use commands::tag::Tagger;

use lalafell::bot::Bot;
use lalafell::commands::prelude::*;

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
      bot
    }
  }
}

impl HasBot for AutoTagCommand {
  fn bot(&self) -> &Bot {
    self.bot.as_ref()
  }
}

#[derive(Debug, Deserialize)]
pub struct Params {
  server: String,
  name: [String; 2]
}

impl HasParams for AutoTagCommand {
  type Params = Params;
}

impl<'a> PublicChannelCommand<'a> for AutoTagCommand {
  fn run(&self, message: &Message, server: &LiveServer, _: &PublicChannel, params: &[&str]) -> CommandResult<'a> {
    let params = self.params(USAGE, params)?;
    let ff_server = params.server;
    let name = params.name.join(" ");

    match Tagger::search_tag(self.bot.as_ref(), message.author.id, server, &ff_server, &name, false)? {
      Some(error) => Err(ExternalCommandFailure::default()
        .message(move |e: EmbedBuilder| e.description(&error))
        .wrap()),
      None => Ok(CommandSuccess::default())
    }
  }
}
