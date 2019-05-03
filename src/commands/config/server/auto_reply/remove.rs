use crate::database::models::ToU64;

use diesel::prelude::*;

use lalafell::commands::prelude::*;
use lalafell::error::*;

use serenity::model::id::GuildId;

pub struct RemoveCommand;

#[derive(Debug, StructOpt)]
pub struct Params {
  #[structopt(help = "The ID of the auto reply to remove")]
  id: i32
}

impl<'a> RemoveCommand {
  #[allow(clippy::needless_pass_by_value)]
  pub fn run(&self, guild: GuildId, params: Params) -> CommandResult<'a> {
    let affected = crate::bot::with_connection(|c| {
      use crate::database::schema::auto_replies::dsl;
      diesel::delete(
        dsl::auto_replies.filter(dsl::id.eq(params.id).and(dsl::server_id.eq(guild.to_u64())))
      )
        .execute(c)
    }).chain_err(|| "could not delete auto_replies")?;
    if affected > 0 {
      Ok(CommandSuccess::default())
    } else {
      Err("No auto replies were deleted.".into())
    }
  }
}
