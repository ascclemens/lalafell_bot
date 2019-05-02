use database::models::{ToU64, AutoReply};

use diesel::prelude::*;

use lalafell::commands::prelude::*;
use lalafell::error::*;

use serenity::prelude::Mentionable;
use serenity::model::id::{GuildId, ChannelId};

pub struct ListCommand;

impl<'a> ListCommand {
  pub fn run(&self, _: &Context, guild: GuildId) -> CommandResult<'a> {
    Ok(ListCommand::list_all(guild)?.into())
  }

  fn list_all(guild: GuildId) -> Result<String> {
    let ars: Vec<AutoReply> = ::bot::with_connection(|c| {
      use database::schema::auto_replies::dsl;
      dsl::auto_replies
        .filter(dsl::server_id.eq(guild.to_u64()))
        .load(c)
    }).chain_err(|| "could not load auto_replies")?;
    Ok(ars.iter()
      .map(|r| format!("{id}. Replying to messages in {channel}{filters} with a delay of {delay} second{plural}.\n```{message}\n```",
                      id = r.id,
                      channel = ChannelId(*r.channel_id).mention(),
                      filters = r.filters.as_ref().map(|f| format!(" with filters `{}`", f)).unwrap_or_default(),
                      delay = r.delay,
                      plural = if r.delay == 1 { "" } else { "s" },
                      message = r.message
      ))
      .collect::<Vec<_>>()
      .join("\n"))
  }
}
