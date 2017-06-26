use bot::LalafellBot;
use commands::*;

use discord::builders::EmbedBuilder;
use discord::model::{LiveServer, PublicChannel, UserId};

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

impl HasBot for ViewTagCommand {
  fn bot(&self) -> Arc<LalafellBot> {
    self.bot.clone()
  }
}

#[derive(Debug, Deserialize)]
pub struct Params {
  who: String
}

impl HasParams for ViewTagCommand {
  type Params = Params;
}

impl<'a> PublicChannelCommand<'a> for ViewTagCommand {
  fn run(&self, message: &Message, _: &LiveServer, channel: &PublicChannel, params: &[&str]) -> CommandResult<'a> {
    let params = self.params(USAGE, params)?;
    let server_id = channel.server_id;
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

    let database = self.bot.database.read().unwrap();
    let user = database.autotags.users.iter().find(|u| u.user_id == who.0 && u.server_id == server_id.0);

    let msg = match user {
      Some(u) => format!("{} is {} on {}.", who.mention(), u.character, u.server),
      None => format!("{} is not tagged.", who.mention())
    };
    Ok(msg.into())
  }
}
