use database::models::{Reaction, NewReaction};

use discord::model::{permissions, UserId, ChannelId, LiveServer};

use diesel;
use diesel::prelude::*;

use lalafell::error::*;
use lalafell::commands::prelude::*;
use lalafell::commands::ChannelOrId;

pub fn reaction<'a>(author: UserId, server: &LiveServer, args: &[String]) -> CommandResult<'a> {
  if !server.permissions_for(server.id.main(), author).contains(permissions::MANAGE_ROLES) {
    return Err(ExternalCommandFailure::default()
      .message(|e: EmbedBuilder| e
        .title("Not enough permissions.")
        .description("You don't have enough permissions to use this command."))
      .wrap());
  }
  if args.len() < 2 {
    let reactions: Vec<Reaction> = ::bot::CONNECTION.with(|c| {
      use database::schema::reactions::dsl;
      dsl::reactions.load(c).chain_err(|| "could not load reactions")
    })?;
    let strings: Vec<String> = reactions.iter()
      .map(|r| format!("{}. {} grants `{}` on {} in {}", r.id, r.emoji, r.role, *r.message_id, ChannelId(*r.channel_id).mention()))
      .collect();
    return Ok(strings.join("\n").into());
  }
  match args[1].to_lowercase().as_str() {
    "add" | "create" => {
      if args.len() < 6 {
        return Err("!configure server reaction add [channel] [emoji] [messageID] [role]".into());
      }
      let channel = ChannelOrId::parse(&args[2]).map_err(|_| into!(CommandFailure, "Invalid channel reference."))?;
      let emoji = &args[3];
      let message_id: u64 = args[4].parse().map_err(|_| into!(CommandFailure, "Invalid message ID."))?;
      let role = args[5..].join(" ").to_lowercase();
      let role = match server.roles.iter().find(|r| r.name.to_lowercase() == role) {
        Some(r) => r.name.clone(),
        None => return Err("No such role.".into())
      };
      let new_reaction = NewReaction {
        server_id: server.id.into(),
        channel_id: channel.into(),
        message_id: message_id.into(),
        emoji: emoji.to_string(),
        role
      };
      ::bot::CONNECTION.with(|c| {
        diesel::insert_into(::database::schema::reactions::table)
          .values(&new_reaction)
          .execute(c)
          .chain_err(|| "could not insert reaction")
      })?;
      Ok(CommandSuccess::default())
    },
    "remove" | "delete" => {
      if args.len() < 3 {
        return Err("!configure server reaction remove [id]".into());
      }
      let id: i32 = args[2].parse().map_err(|_| into!(CommandFailure, "Invalid ID."))?;
      let affected = ::bot::CONNECTION.with(|c| {
        use database::schema::reactions::dsl;
        diesel::delete(dsl::reactions.filter(dsl::id.eq(id)))
          .execute(c)
          .chain_err(|| "could not delete reaction")
      })?;
      if affected > 0 {
        Ok(CommandSuccess::default())
      } else {
        Err("No reactions were deleted.".into())
      }
    },
    _ => Err("Invalid subcommand.".into())
  }
}
