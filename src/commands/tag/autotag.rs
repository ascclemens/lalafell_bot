use LalafellBot;
use commands::*;
use commands::tag::Tagger;

use discord::builders::EmbedBuilder;
use discord::model::Channel;

use xivdb::error::*;

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

impl<'a> Command<'a> for AutoTagCommand {
  fn run(&self, message: &Message, params: &[&str]) -> CommandResult<'a> {
    let channel = self.bot.discord.get_channel(message.channel_id).chain_err(|| "could not get channel for message")?;
    let server_id = match channel {
      Channel::Public(c) => c.server_id,
      _ => {
        let err: error::Error = "channel was not public".into();
        return Err(err.into());
      }
    };
    let mut state_option = self.bot.state.lock().unwrap();
    let state = state_option.as_mut().unwrap();
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

    match Tagger::search_tag(self.bot.clone(), message.author.id, server, ff_server, &name)? {
      Some(error) => Err(ExternalCommandFailure::default()
        .message(move |e: EmbedBuilder| e.description(&error))
        .wrap()),
      None => Ok(CommandSuccess::default())
    }
  }
}
