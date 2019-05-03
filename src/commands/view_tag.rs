use crate::{
  commands::*,
  database::models::{ToU64, Tag},
};

use lalafell::{
  commands::prelude::*,
  error::*,
};

use serenity::prelude::Mentionable;

use diesel::prelude::*;

use std::sync::Arc;

#[derive(BotCommand)]
pub struct ViewTagCommand;

#[derive(Debug, StructOpt)]
#[structopt(about = "View the FFXIV character tag of a member")]
pub struct Params {
  #[structopt(help = "Who to view the tag of")]
  who: MentionOrId,
}

impl HasParams for ViewTagCommand {
  type Params = Params;
}

impl<'a> PublicChannelCommand<'a> for ViewTagCommand {
  fn run(&self, _: &Context, _: &Message, guild: GuildId, _: Arc<RwLock<GuildChannel>>, params: &[&str]) -> CommandResult<'a> {
    let params = self.params_then("viewtag", params, |a| a.setting(structopt::clap::AppSettings::ArgRequiredElseHelp))?;
    let who = params.who;

    let tag: Option<Tag> = crate::bot::with_connection(|c| {
      use crate::database::schema::tags::dsl;
      dsl::tags
        .filter(dsl::user_id.eq(who.to_u64()).and(dsl::server_id.eq(guild.to_u64())))
        .first(c)
        .optional()
    }).chain_err(|| "could not load tags")?;

    let msg = match tag {
      Some(u) => format!("{} is {} on {}.", who.mention(), u.character, u.server),
      None => format!("{} is not tagged.", who.mention()),
    };
    Ok(msg.into())
  }
}
