use filters::Filter;

use lalafell::commands::prelude::*;
use lalafell::commands::ChannelOrId;

use serenity::prelude::Mentionable;
use serenity::model::guild::{Role, Member};

use rand::{thread_rng, Rng};

use std::sync::Arc;

const USAGE: &str = "!randomreaction <channel/id> <message id> <emoji> [filters]";

#[derive(Default)]
pub struct RandomReactionCommand;

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
    let mut reactions = params.channel.reaction_users(params.message_id, params.emoji.as_str(), Some(100), None)
      .map_err(|_| into!(CommandFailure, "Could not get reactions."))?;
    if reactions.is_empty() {
      return Err("No reactions on that message.".into());
    }
    loop {
      let last_reaction = reactions[reactions.len() - 1].id;
      let next_batch = params.channel.reaction_users(params.message_id, params.emoji.as_str(), Some(100), last_reaction)
      .map_err(|_| into!(CommandFailure, "Could not get reactions."))?;
      if next_batch.is_empty() {
        break;
      }
      reactions.extend(next_batch);
    }
    let filters = match Filter::all_filters(&params.filters.unwrap_or_default().join(" ")) {
      Some(f) => f,
      None => return Err("Invalid filters.".into())
    };
    let guild = some_or!(guild.find(), bail!("could not find guild"));
    let reader = guild.read();
    let roles: Vec<&Role> = reader.roles.values().collect();
    let members: Vec<&Member> = reader.members.values().collect();
    let members: Vec<&&Member> = reactions.into_iter()
      .filter_map(|u| members.iter().find(|m| m.user.read().id == u.id))
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
