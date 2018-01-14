use database::models::{Reaction, NewReaction};

use serenity::prelude::Mentionable;
use serenity::model::id::{ChannelId, UserId};

use diesel;
use diesel::prelude::*;

use lalafell::error::*;
use lalafell::commands::prelude::*;
use lalafell::commands::ChannelOrId;

pub fn reaction<'a>(author: UserId, guild: GuildId, args: &[String]) -> CommandResult<'a> {
  let member = guild.member(author).chain_err(|| "could not get member")?;
  let guild = some_or!(guild.find(), bail!("could not find guild"));
  if !member.permissions().chain_err(|| "could not get permissions")?.manage_roles() {
    return Err(ExternalCommandFailure::default()
      .message(|e: CreateEmbed| e
        .title("Not enough permissions.")
        .description("You don't have enough permissions to use this command."))
      .wrap());
  }
  if args.len() < 1 {
    let reactions: Vec<Reaction> = ::bot::CONNECTION.with(|c| {
      use database::schema::reactions::dsl;
      dsl::reactions.load(c).chain_err(|| "could not load reactions")
    })?;
    let strings: Vec<String> = reactions.iter()
      .map(|r| format!("{}. {} grants `{}` on {} in {}", r.id, r.emoji, r.role, *r.message_id, ChannelId(*r.channel_id).mention()))
      .collect();
    return Ok(strings.join("\n").into());
  }
  match args[0].to_lowercase().as_str() {
    "add" | "create" => {
      if args.len() < 5 {
        return Err("!configure server reaction add [channel] [emoji] [messageID] [role]".into());
      }
      let channel = ChannelOrId::parse(&args[1]).map_err(|_| into!(CommandFailure, "Invalid channel reference."))?;
      let emoji = &args[2];
      let message_id: u64 = args[3].parse().map_err(|_| into!(CommandFailure, "Invalid message ID."))?;
      let role = args[4..].join(" ").to_lowercase();
      let role = match guild.read().roles.values().find(|r| r.name.to_lowercase() == role) {
        Some(r) => r.name.clone(),
        None => return Err("No such role.".into())
      };
      let new_reaction = NewReaction {
        server_id: guild.read().id.into(),
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
      if args.len() < 2 {
        return Err("!configure server reaction remove [id]".into());
      }
      let id: i32 = args[1].parse().map_err(|_| into!(CommandFailure, "Invalid ID."))?;
      let affected = ::bot::CONNECTION.with(|c| {
        use database::schema::reactions::dsl;
        diesel::delete(dsl::reactions.find(id))
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
