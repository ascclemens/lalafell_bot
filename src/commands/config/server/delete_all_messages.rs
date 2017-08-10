use database::models::{DeleteAllMessages, NewDeleteAllMessages};

use discord::model::{permissions, UserId, ChannelId, LiveServer};

use diesel;
use diesel::prelude::*;

use lalafell::error::*;
use lalafell::commands::prelude::*;
use lalafell::commands::ChannelOrId;

#[derive(Debug, Deserialize)]
pub struct DeleteParams {
  id: i32
}

#[derive(Debug, Deserialize)]
pub struct AddParams {
  channel: ChannelOrId,
  after: u16,
  except: Option<Vec<u64>>
}

fn list_all() -> Result<String> {
  let dams: Vec<DeleteAllMessages> = ::bot::CONNECTION.with(|c| {
    use database::schema::delete_all_messages::dsl;
    dsl::delete_all_messages.load(c).chain_err(|| "could not load delete_all_messages")
  })?;
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

pub fn delete_all_messages<'a>(author: UserId, server: &LiveServer, args: &[String]) -> CommandResult<'a> {
  if !server.permissions_for(server.id.main(), author).contains(permissions::MANAGE_MESSAGES) {
    return Err(ExternalCommandFailure::default()
      .message(|e: EmbedBuilder| e
        .title("Not enough permissions.")
        .description("You don't have enough permissions to use this command."))
      .wrap());
  }
  if args.is_empty() {
    return Ok(list_all()?.into());
  }
  let subcommand = &args[0];
  let args = &args[1..];
  match subcommand.to_lowercase().as_str() {
    "add" | "create" => {
      let params: AddParams = ::lalafell::commands::params::from_str(&args.join(" "))
        .map_err(|_| into!(CommandFailure, "Invalid parameters."))?;
      let dams: Vec<DeleteAllMessages> = ::bot::CONNECTION.with(|c| {
        use database::schema::delete_all_messages::dsl;
        dsl::delete_all_messages
          .filter(dsl::channel_id.eq(params.channel.0.to_string())
            .and(dsl::server_id.eq(server.id.0.to_string())))
          .load(c)
          .chain_err(|| "could not load delete_all_messages")
      })?;
      if !dams.is_empty() {
        return Err("A delete all messages already exists for that channel.".into());
      }

      let ndam = NewDeleteAllMessages::new(server.id.0, params.channel.0, params.after as i32, &params.except.unwrap_or_default());
      ::bot::CONNECTION.with(|c| {
        use database::schema::delete_all_messages;
        diesel::insert(&ndam)
          .into(delete_all_messages::table)
          .execute(c)
          .chain_err(|| "could not insert new dam")
      })?;
      Ok(CommandSuccess::default())
    },
    "remove" | "delete" => {
      let params: DeleteParams = ::lalafell::commands::params::from_str(&args.join(" "))
        .map_err(|_| into!(CommandFailure, "Invalid parameters."))?;
      let affected = ::bot::CONNECTION.with(|c| {
        use database::schema::delete_all_messages::dsl;
        diesel::delete(dsl::delete_all_messages.filter(dsl::id.eq(params.id)))
          .execute(c)
          .chain_err(|| "could not delete delete_all_messages")
      })?;
      if affected > 0 {
        Ok(CommandSuccess::default())
      } else {
        Err("No delete all messages were deleted.".into())
      }
    },
    _ => Err("No such subcommand.".into())
  }
}
