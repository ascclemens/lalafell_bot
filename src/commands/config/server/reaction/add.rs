use crate::database::models::NewReaction;
use crate::util::ParsedEmoji;

use diesel::prelude::*;

use lalafell::commands::ChannelOrId;
use lalafell::commands::prelude::*;
use lalafell::error::*;

use serenity::model::id::GuildId;

use unicase::UniCase;

pub struct AddCommand;

#[derive(Debug, StructOpt)]
pub struct Params {
  #[structopt(help = "The channel to add the reaction role to")]
  channel: ChannelOrId,
  #[structopt(help = "The emoji to trigger the reaction role")]
  #[structopt(parse(from_str))]
  emoji: ParsedEmoji,
  #[structopt(help = "The message ID of the message to add the reaction role to")]
  message_id: u64,
  #[structopt(help = "The name of the role to add")]
  #[structopt(use_delimiter = false)]
  role: String
}

impl<'a> AddCommand {
  #[allow(clippy::needless_pass_by_value)]
  pub fn run(&self, ctx: &Context, guild_id: GuildId, params: Params) -> CommandResult<'a> {
    let guild = guild_id.to_guild_cached(&ctx).chain_err(|| "could not find guild")?;
    let role = UniCase::new(params.role);
    let role_id = match guild.read().roles.values().find(|r| UniCase::new(&r.name) == role) {
      Some(r) => r.id,
      None => return Err("No such role.".into())
    };
    let new_reaction = NewReaction {
      server_id: guild_id.into(),
      channel_id: (*params.channel).into(),
      message_id: params.message_id.into(),
      emoji: params.emoji.to_string(),
      role_id: role_id.into()
    };
    crate::bot::with_connection(|c| {
      diesel::insert_into(crate::database::schema::reactions::table)
        .values(&new_reaction)
        .execute(c)
    }).chain_err(|| "could not insert reaction")?;
    Ok(CommandSuccess::default())
  }
}
