use database::models::{AutoReply, NewAutoReply};
use filters::Filter;

use serenity::prelude::Mentionable;
use serenity::builder::CreateEmbed;
use serenity::model::id::{UserId, ChannelId};

use diesel;
use diesel::prelude::*;

use lalafell::error::*;
use lalafell::commands::prelude::*;
use lalafell::commands::ChannelOrId;

#[derive(Debug, Deserialize)]
pub struct DeleteParams {
  id: i32
}

fn list_all() -> Result<String> {
  let ars: Vec<AutoReply> = ::bot::CONNECTION.with(|c| {
    use database::schema::auto_replies::dsl;
    dsl::auto_replies.load(c).chain_err(|| "could not load auto_replies")
  })?;
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

pub fn auto_reply<'a>(author: UserId, guild: GuildId, content: &str) -> CommandResult<'a> {
  let member = guild.member(author).chain_err(|| "could not get member")?;
  if !member.permissions().chain_err(|| "could not get permissions")?.manage_messages() {
    return Err(ExternalCommandFailure::default()
      .message(|e: CreateEmbed| e
        .title("Not enough permissions.")
        .description("You don't have enough permissions to use this command."))
      .wrap());
  }
  let args: Vec<&str> = content.split(|c| c == ' ' || c == '\t').filter(|x| !x.is_empty()).skip(3).collect();
  if args.is_empty() {
    return Ok(list_all()?.into());
  }
  let subcommand = &args[0];
  let args = &args[1..];
  match subcommand.to_lowercase().as_str() {
    "add" | "create" => {
      if args.len() < 4 {
        return Err("Invalid parameters.".into());
      }
      let channel = ChannelOrId::parse(args[0]).map_err(|_| into!(CommandFailure, "Invalid channel reference."))?;
      let on_join = match args[1].to_lowercase().as_str() {
        "true" | "yes" | "y" => true,
        "false" | "no" | "n" => false,
        _ => return Err("Invalid on-join parameter.".into())
      };
      let (message, delay_str) = if args[2].contains('\n') {
        let joined = args[2..].join(" ");
        let parts: Vec<&str> = joined.splitn(2, '\n').collect();
        assert_eq!(2, parts.len()); // contains \n, so splitn should always return two parts
        (Some(parts[1].to_string()), parts[0].to_string())
      } else {
        (None, args[2].to_string())
      };
      let delay: u64 = match delay_str.parse() {
        Ok(d) => d,
        Err(_) => return Err("Invalid delay.".into())
      };
      let (filters, message) = match message {
        Some(m) => (None, m.to_string()),
        None => {
          let args = &args[3..];
          let joined_args = args.join(" ");
          let parts: Vec<&str> = joined_args.splitn(2, '\n').collect();
          if parts.len() == 1 {
            return Err("Missing message.".into());
          }
          let filters = match Filter::all_filters(parts[0]) {
            Some(f) => f.into_iter().map(|x| x.to_string()).collect::<Vec<_>>().join(" "),
            None => return Err("Invalid filters.".into())
          };
          (Some(filters), parts[1].to_string())
        }
      };
      let nar = NewAutoReply {
        server_id: guild.into(),
        channel_id: channel.0.into(),
        message: message.to_string(),
        on_join,
        delay: delay as i32,
        filters
      };
      ::bot::CONNECTION.with(|c| {
        use database::schema::auto_replies;
        diesel::insert_into(auto_replies::table)
          .values(&nar)
          .execute(c)
          .chain_err(|| "could not insert new dam")
      })?;
      Ok(CommandSuccess::default())
    },
    "remove" | "delete" => {
      let params: DeleteParams = ::lalafell::commands::params::from_str(&args.join(" "))
        .map_err(|_| into!(CommandFailure, "Invalid parameters."))?;
      let affected = ::bot::CONNECTION.with(|c| {
        use database::schema::auto_replies::dsl;
        diesel::delete(dsl::auto_replies.filter(dsl::id.eq(params.id)))
          .execute(c)
          .chain_err(|| "could not delete auto_replies")
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
