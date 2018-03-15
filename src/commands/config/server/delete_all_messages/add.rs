use database::models::{ToU64, DeleteAllMessages, NewDeleteAllMessages};

use diesel::prelude::*;

use lalafell::commands::ChannelOrId;
use lalafell::commands::prelude::*;
use lalafell::error::*;

use serenity::model::id::GuildId;

pub struct AddCommand;

#[derive(Debug, StructOpt)]
pub struct Params {
  // FIXME: Take String and check against voice channels
  #[structopt(help = "The channel to add the DAM to")]
  channel: ChannelOrId,
  #[structopt(help = "The number of seconds after which a message is posted before it becomes eligible to be deleted")]
  after: u16,
  #[structopt(help = "The list of message IDs not to delete")]
  except: Vec<u64>
}

impl<'a> AddCommand {
  #[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
  pub fn run(&self, guild: GuildId, params: Params) -> CommandResult<'a> {
    let dams: Vec<DeleteAllMessages> = ::bot::with_connection(|c| {
      use database::schema::delete_all_messages::dsl;
      dsl::delete_all_messages
        .filter(dsl::channel_id.eq(params.channel.to_u64())
          .and(dsl::server_id.eq(guild.to_u64())))
        .load(c)
    }).chain_err(|| "could not load delete_all_messages")?;
    if !dams.is_empty() {
      return Err("A delete all messages already exists for that channel.".into());
    }

    let ndam = NewDeleteAllMessages::new(guild.0, params.channel.0, i32::from(params.after), &params.except);
    ::bot::with_connection(|c| {
      use database::schema::delete_all_messages;
      ::diesel::insert_into(delete_all_messages::table)
        .values(&ndam)
        .execute(c)
    }).chain_err(|| "could not insert new dam")?;

    Ok(CommandSuccess::default())
  }
}
