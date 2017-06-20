use bot::LalafellBot;
use commands::*;
use commands::tag::Tagger;

use discord::builders::EmbedBuilder;
use discord::model::PublicChannel;

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
  fn run(&self, message: &Message, channel: &PublicChannel, params: &[&str]) -> CommandResult<'a> {
    let server_id = channel.server_id;
    let state_option = self.bot.state.read().unwrap();
    let state = state_option.as_ref().unwrap();
    let server = match state.servers().iter().find(|x| x.id == server_id) {
      Some(s) => s,
      None => {
        let err: error::Error = "could not find server for channel".into();
        return Err(err.into());
      }
    };

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
