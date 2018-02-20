use database::models::NewAutoReply;
use filters::Filter;
use util::ParsedDuration;

use diesel::prelude::*;

use lalafell::commands::ChannelOrId;
use lalafell::commands::prelude::*;
use lalafell::error::*;

use serenity::model::id::GuildId;

pub struct AddCommand;

#[derive(Debug, StructOpt)]
pub struct Params {
  #[structopt(short = "j", long = "on-join", help = "If the message should be sent on join")]
  on_join: bool,

  #[structopt(short = "d", long = "delay", help = "The time string for how long to wait before sending the message again")]
  #[structopt(parse(try_from_str))]
  delay: Option<ParsedDuration>,

  #[structopt(short = "f", long = "filter", help = "A filter to add to this auto reply")]
  #[structopt(raw(number_of_values = "1"))]
  filters: Vec<String>,

  #[structopt(help = "The channel to add the auto reply to")]
  channel: ChannelOrId,

  // FIXME: Handle newlines. Probably do a party-finder-like interface
  #[structopt(help = "The message to send")]
  #[structopt(raw(use_delimiter = "false"))]
  message: Vec<String>
}

impl<'a> AddCommand {
  pub fn run(&self, guild: GuildId, params: Params) -> CommandResult<'a> {
    let filters = if params.filters.is_empty() {
      None
    } else {
      match Filter::all_filters(&params.filters.join(" ")) {
        Some(f) => Some(f.into_iter().map(|x| x.to_string()).collect::<Vec<_>>().join(" ")),
        None => return Err("Invalid filters.".into())
      }
    };
    let delay: i32 = params.delay.unwrap_or_default().0 as i32;
    let message = params.message.join(" ");
    if message.is_empty() {
      return Err("Empty message.".into());
    }
    let nar = NewAutoReply {
      server_id: guild.into(),
      channel_id: params.channel.0.into(),
      message,
      on_join: params.on_join,
      delay,
      filters
    };
    ::bot::CONNECTION.with(|c| {
      use database::schema::auto_replies;
      ::diesel::insert_into(auto_replies::table)
        .values(&nar)
        .execute(c)
        .chain_err(|| "could not insert new dam")
    })?;
    Ok(CommandSuccess::default())
  }
}
