use bot::LalafellBot;
use commands::*;
use database::models::Tag;

use lalafell::bot::Bot;
use lalafell::commands::prelude::*;
use lalafell::error::*;

use discord::model::{Message, LiveServer, PublicChannel};

use diesel::prelude::*;

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

    let tag: Option<Tag> = ::bot::CONNECTION.with(|c| {
      use database::schema::tags::dsl;
      dsl::tags
        .filter(dsl::user_id.eq(who.0.to_string()).and(dsl::server_id.eq(server_id.0.to_string())))
        .first(c)
        .optional()
        .chain_err(|| "could not load tags")
    })?;

    let msg = match tag {
      Some(u) => format!("{} is {} on {}.", who.mention(), u.character, u.server),
      None => format!("{} is not tagged.", who.mention())
    };
    Ok(msg.into())
  }
}
