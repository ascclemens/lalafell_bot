use database::models::{ToU64, DeleteAllMessages};

use diesel::prelude::*;

use lalafell::commands::prelude::*;
use lalafell::error::*;

use serenity::prelude::Mentionable;
use serenity::model::id::{GuildId, ChannelId};

pub struct ListCommand;

impl<'a> ListCommand {
  pub fn run(&self, guild: GuildId) -> CommandResult<'a> {
    Ok(ListCommand::list_all(guild)?.into())
  }

  fn list_all(guild: GuildId) -> Result<String> {
    let dams: Vec<DeleteAllMessages> = ::bot::with_connection(|c| {
      use database::schema::delete_all_messages::dsl;
      dsl::delete_all_messages
        .filter(dsl::server_id.eq(guild.to_u64()))
        .load(c)
    }).chain_err(|| "could not load delete_all_messages")?;
    Ok(dams.iter()
      .map(|d| format!("{id}. Deleting all messages in {channel} after {after} second{plural}{except}.",
                      id = d.id,
                      channel = ChannelId(*d.channel_id).mention(),
                      after = d.after,
                      plural = if d.after == 1 { "" } else { "s" },
                      except = if d.exclude.is_empty() { String::new() } else { format!(" (excluding {} message{})", d.exclude.len() / 8, if d.exclude.len() / 8 == 1 { "" } else { "s" }) }
      ))
      .collect::<Vec<_>>()
      .join("\n"))
  }
}
