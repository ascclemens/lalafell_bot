use database::models::{ToU64, Reaction};

use diesel::prelude::*;

use lalafell::commands::prelude::*;
use lalafell::error::*;

use serenity::prelude::Mentionable;
use serenity::model::id::{GuildId, ChannelId, RoleId};

pub struct ListCommand;

impl<'a> ListCommand {
  pub fn run(&self, guild: GuildId) -> CommandResult<'a> {
    let reactions: Vec<Reaction> = ::bot::with_connection(|c| {
      use database::schema::reactions::dsl;
      dsl::reactions
        .filter(dsl::server_id.eq(guild.to_u64()))
        .load(c)
    }).chain_err(|| "could not load reactions")?;
    let strings: Vec<String> = reactions.iter()
      .map(|r| format!("{}. {} grants {} on {} in {}", r.id, r.emoji, RoleId(*r.role_id).mention(), *r.message_id, ChannelId(*r.channel_id).mention()))
      .collect();
    Ok(strings.join("\n").into())
  }
}
