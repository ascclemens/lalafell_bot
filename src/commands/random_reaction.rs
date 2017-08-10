use bot::LalafellBot;
use filters::Filter;

use lalafell::bot::Bot;
use lalafell::commands::prelude::*;
use lalafell::commands::ChannelOrId;

use discord::model::{Message, LiveServer, PublicChannel, MessageId, ReactionEmoji, Member};

use rand::{thread_rng, Rng};

use std::sync::Arc;

const USAGE: &'static str = "!randomreaction <channel/id> <message id> <emoji> [filters]";

pub struct RandomReactionCommand {
  bot: Arc<LalafellBot>
}

impl RandomReactionCommand {
  pub fn new(bot: Arc<LalafellBot>) -> RandomReactionCommand {
    RandomReactionCommand {
      bot: bot
    }
  }
}

impl HasBot for RandomReactionCommand {
  fn bot(&self) -> &Bot {
    self.bot.as_ref()
  }
}

#[derive(Debug, Deserialize)]
pub struct Params {
  channel: ChannelOrId,
  message_id: u64,
  emoji: String,
  filters: Option<Vec<String>>
}

impl HasParams for RandomReactionCommand {
  type Params = Params;
}

impl<'a> PublicChannelCommand<'a> for RandomReactionCommand {
  fn run(&self, _: &Message, server: &LiveServer, _: &PublicChannel, params: &[&str]) -> CommandResult<'a> {
    let params = self.params(USAGE, params)?;
    let reactions = self.bot.discord.get_reactions(*params.channel, MessageId(params.message_id), ReactionEmoji::Unicode(params.emoji), Some(100), None)
      .map_err(|_| into!(CommandFailure, "Could not get reactions."))?;
    let filters: Option<Vec<Filter>> = params.filters.unwrap_or_default().iter().map(|x| Filter::parse(x)).collect();
    let filters = match filters {
      Some(f) => f,
      None => return Err("Invalid filters.".into())
    };
    let members: Vec<&Member> = reactions.into_iter()
      .map(|u| server.members.iter().find(|m| m.user.id == u.id))
      .filter(Option::is_some)
      .map(Option::unwrap)
      .filter(|m| filters.iter().all(|f| f.matches(m, &server.roles)))
      .collect();
    if members.is_empty() {
      return Err("No reactions matched those criteria.".into());
    }
    if members.len() == 1 {
      return Ok(format!("Only {} matched those criteria!", members[0].user.mention()).into());
    }
    let member = match thread_rng().choose(&members) {
      Some(u) => u,
      None => return Err("Could not randomly choose a reaction.".into())
    };
    Ok(format!("I've randomly selected {}!", member.user.mention()).into())
  }
}
