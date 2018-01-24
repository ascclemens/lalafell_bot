use commands::*;
use database::models::{ToU64, Tag};

use lalafell::commands::prelude::*;
use lalafell::error::*;

use serenity::prelude::Mentionable;

use diesel::prelude::*;

use std::sync::Arc;

const USAGE: &str = "!viewtag <who>";

#[derive(BotCommand)]
pub struct ViewTagCommand;

#[derive(Debug, Deserialize)]
pub struct Params {
  who: MentionOrId
}

impl HasParams for ViewTagCommand {
  type Params = Params;
}

impl<'a> PublicChannelCommand<'a> for ViewTagCommand {
  fn run(&self, _: &Context, _: &Message, guild: GuildId, _: Arc<RwLock<GuildChannel>>, params: &[&str]) -> CommandResult<'a> {
    let params = self.params(USAGE, params)?;
    let who = params.who;

    let tag: Option<Tag> = ::bot::CONNECTION.with(|c| {
      use database::schema::tags::dsl;
      dsl::tags
        .filter(dsl::user_id.eq(who.to_u64()).and(dsl::server_id.eq(guild.to_u64())))
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
