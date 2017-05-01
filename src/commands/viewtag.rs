use LalafellBot;
use commands::*;

use discord::builders::EmbedBuilder;
use discord::model::{Channel, UserId};

use xivdb::error::*;

use std::sync::Arc;

const USAGE: &'static str = "!viewtag <who>";

pub struct ViewTagCommand {
  bot: Arc<LalafellBot>
}

impl ViewTagCommand {
  pub fn new(bot: Arc<LalafellBot>) -> ViewTagCommand {
    ViewTagCommand {
      bot: bot
    }
  }
}

impl<'a> Command<'a> for ViewTagCommand {
  fn run(&self, message: &Message, params: &[&str]) -> CommandResult<'a> {
    if params.is_empty() {
      return Err(ExternalCommandFailure::default()
        .message(|e: EmbedBuilder| e
          .title("Not enough parameters.")
          .description(USAGE))
        .wrap());
    }
    let channel = self.bot.discord.get_channel(message.channel_id).chain_err(|| "could not get channel for message")?;
    let server_id = match channel {
      Channel::Public(c) => c.server_id,
      _ => {
        let err: error::Error = "channel was not public".into();
        return Err(err.into());
      }
    };
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

    let database = self.bot.database.lock().unwrap();
    let user = database.autotags.users.iter().find(|u| u.user_id == who.0 && u.server_id == server_id.0);

    let msg = match user {
      Some(u) => format!("{} is {} on {}.", who.mention(), u.character, u.server),
      None => format!("{} is not tagged.", who.mention())
    };
    Ok(CommandSuccess::default()
      .message(move |e: EmbedBuilder| e.description(&msg)))
  }
}
