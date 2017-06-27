use bot::LalafellBot;
use commands::*;

use lalafell::bot::Bot;
use lalafell::commands::prelude::*;

use discord::model::{Message, LiveServer, PublicChannel};

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
  fn bot(&self) -> &Bot {
    self.bot.as_ref()
  }
}

#[derive(Debug, Deserialize)]
pub struct Params {
  who: MentionOrId
}

impl HasParams for ViewTagCommand {
  type Params = Params;
}

impl<'a> PublicChannelCommand<'a> for ViewTagCommand {
  fn run(&self, _: &Message, _: &LiveServer, channel: &PublicChannel, params: &[&str]) -> CommandResult<'a> {
    let params = self.params(USAGE, params)?;
    let server_id = channel.server_id;
    let who = params.who;

    let database = self.bot.database.read().unwrap();
    let user = database.autotags.users.iter().find(|u| u.user_id == who.0 && u.server_id == server_id.0);

    let msg = match user {
      Some(u) => format!("{} is {} on {}.", who.mention(), u.character, u.server),
      None => format!("{} is not tagged.", who.mention())
    };
    Ok(msg.into())
  }
}
