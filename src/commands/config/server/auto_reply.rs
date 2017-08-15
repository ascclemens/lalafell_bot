use database::models::{AutoReply, NewAutoReply};
use filters::Filter;

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

fn list_all() -> Result<String> {
  let ars: Vec<AutoReply> = ::bot::CONNECTION.with(|c| {
    use database::schema::auto_replies::dsl;
    dsl::auto_replies.load(c).chain_err(|| "could not load auto_replies")
  })?;
  Ok(ars.iter()
    .map(|r| format!("{id}. Replying to messages in {channel}{filters} with a delay of {delay} second{plural}.\n```{message}```",
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

pub fn auto_reply<'a>(author: UserId, server: &LiveServer, content: &str) -> CommandResult<'a> {
  if !server.permissions_for(server.id.main(), author).contains(permissions::MANAGE_MESSAGES) {
    return Err(ExternalCommandFailure::default()
      .message(|e: EmbedBuilder| e
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
          let filters: Option<Vec<Filter>> = parts[0].split(' ').map(Filter::parse).collect();
          let filters = match filters {
            Some(f) => f.into_iter().map(|x| x.to_string()).collect::<Vec<_>>().join(" "),
            None => return Err("Invalid filters.".into())
          };
          (Some(filters), parts[1].to_string())
        }
      };
      let nar = NewAutoReply {
        server_id: server.id.0.into(),
        channel_id: channel.0.into(),
        message: message.to_string(),
        on_join: on_join,
        delay: delay as i32,
        filters: filters
      };
      ::bot::CONNECTION.with(|c| {
        use database::schema::auto_replies;
        diesel::insert(&nar)
          .into(auto_replies::table)
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
