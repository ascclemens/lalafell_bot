use bot::LalafellBot;
use filters::Filter;

use lalafell::bot::Bot;
use lalafell::commands::prelude::*;

use discord::model::{Message, LiveServer, PublicChannel};
use discord::model::permissions;

use chrono::DateTime;

use std::sync::Arc;

const USAGE: &'static str = "!search <filters>";

pub struct SearchCommand {
  bot: Arc<LalafellBot>
}

impl SearchCommand {
  pub fn new(bot: Arc<LalafellBot>) -> SearchCommand {
    SearchCommand {
      bot
    }
  }
}

impl HasBot for SearchCommand {
  fn bot(&self) -> &Bot {
    self.bot.as_ref()
  }
}

#[derive(Debug, Deserialize)]
pub struct Params {
  filter_strings: Vec<String>
}

impl HasParams for SearchCommand {
  type Params = Params;
}

impl<'a> PublicChannelCommand<'a> for SearchCommand {
  fn run(&self, message: &Message, server: &LiveServer, channel: &PublicChannel, params: &[&str]) -> CommandResult<'a> {
    let params = self.params(USAGE, params)?;

    let can_manage_roles = server.permissions_for(channel.id, message.author.id).contains(permissions::MANAGE_ROLES);
    if !can_manage_roles {
      return Err(ExternalCommandFailure::default()
        .message(|e: EmbedBuilder| e
          .title("Not enough permissions.")
          .description("You don't have enough permissions to use this command."))
        .wrap());
    }

    let filters: Vec<Filter> = match params.filter_strings.iter().map(|x| Filter::parse(x)).collect::<Option<_>>() {
      Some(f) => f,
      None => return Err("Invalid filter.".into())
    };
    let matches: Vec<String> = server.members.iter()
      .filter(|m| filters.iter().all(|f| f.matches(m, &server.roles)))
      .map(|m| format!("{} - {}",
        m.user.mention(),
        DateTime::parse_from_rfc3339(&m.joined_at).map(|d| d.format("%B %e, %Y %H:%M").to_string()).unwrap_or_else(|_| String::from("unknown"))))
      .collect();
    Ok(matches.join("\n").into())
  }
}
