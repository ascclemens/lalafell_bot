use bot::BotEnv;
use filters::Filter;

use lalafell::commands::prelude::*;
use lalafell::commands::ChannelOrId;

use serenity::prelude::Mentionable;
use serenity::model::guild::{Role, Member};
use serenity::model::channel::ReactionType;

use rand::{thread_rng, Rng};

use std::sync::Arc;

const USAGE: &'static str = "!randomreaction <channel/id> <message id> <emoji> [filters]";

pub struct RandomReactionCommand;

impl RandomReactionCommand {
  pub fn new(_: Arc<BotEnv>) -> Self {
    RandomReactionCommand
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
  fn run(&self, _: &Context, _: &Message, guild: GuildId, _: Arc<RwLock<GuildChannel>>, params: &[&str]) -> CommandResult<'a> {
    let params = self.params(USAGE, params)?;
    // FIXME: support > 100
    let reactions = params.channel.reaction_users::<u64, ReactionType, u64>(params.message_id, ReactionType::Unicode(params.emoji), Some(100), None)
      .map_err(|_| into!(CommandFailure, "Could not get reactions."))?;
    let filters: Option<Vec<Filter>> = params.filters.unwrap_or_default().iter().map(|x| Filter::parse(x)).collect();
    let filters = match filters {
      Some(f) => f,
      None => return Err("Invalid filters.".into())
    };
    let guild = match guild.find() {
      Some(g) => g,
      None => return Err("The guild must be cached.".into())
    };
    // FIXME: do less cloning
    let roles: Vec<Role> = guild.read().roles.values().cloned().collect();
    let members: Vec<Member> = guild.read().members.values().cloned().collect();
    let members: Vec<&Member> = reactions.into_iter()
      .map(|u| members.iter().find(|m| m.user.read().id == u.id))
      .filter(Option::is_some)
      .map(Option::unwrap)
      .filter(|m| filters.iter().all(|f| f.matches(m, &roles)))
      .collect();
    if members.is_empty() {
      return Err("No reactions matched those criteria.".into());
    }
    if members.len() == 1 {
      return Ok(format!("Only {} matched those criteria!", members[0].mention()).into());
    }
    let member = match thread_rng().choose(&members) {
      Some(u) => u,
      None => return Err("Could not randomly choose a reaction.".into())
    };
    Ok(format!("I've randomly selected {}!", member.mention()).into())
  }
}
