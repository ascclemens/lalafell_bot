use filters::Filter;
use util::ParsedEmoji;

use lalafell::error::*;
use lalafell::commands::prelude::*;
use lalafell::commands::ChannelOrId;

use serenity::prelude::Mentionable;
use serenity::model::guild::{Role, Member};

use rand::{thread_rng, seq::SliceRandom};

use std::sync::Arc;

#[derive(BotCommand)]
pub struct RandomReactionCommand;

#[derive(Debug, StructOpt)]
#[structopt(about = "Pick a random member who reaction to a message")]
pub struct Params {
  #[structopt(help = "The channel the message is in")]
  channel: ChannelOrId,
  #[structopt(help = "The message ID")]
  message_id: u64,
  #[structopt(help = "The emoji of the reaction")]
  #[structopt(parse(from_str))]
  emoji: ParsedEmoji,
  #[structopt(help = "A list of filters to apply when picking a random member")]
  filters: Vec<String>
}

impl HasParams for RandomReactionCommand {
  type Params = Params;
}

impl<'a> PublicChannelCommand<'a> for RandomReactionCommand {
  fn run(&self, _: &Context, _: &Message, guild: GuildId, _: Arc<RwLock<GuildChannel>>, params: &[&str]) -> CommandResult<'a> {
    let params = self.params_then("randomreaction", params, |a| a.setting(::structopt::clap::AppSettings::ArgRequiredElseHelp))?;
    let emoji = params.emoji;
    let mut reactions = params.channel.reaction_users(params.message_id, emoji.clone(), Some(100), None)
      .map_err(|_| into!(CommandFailure, "Could not get reactions."))?;
    if reactions.is_empty() {
      return Err("No reactions on that message.".into());
    }
    loop {
      let last_reaction = reactions[reactions.len() - 1].id;
      let next_batch = params.channel.reaction_users(params.message_id, emoji.clone(), Some(100), last_reaction)
      .map_err(|_| into!(CommandFailure, "Could not get reactions."))?;
      if next_batch.is_empty() {
        break;
      }
      reactions.extend(next_batch);
    }
    let filters = match Filter::all_filters(&params.filters.join(" ")) {
      Some(f) => f,
      None => return Err("Invalid filters.".into())
    };
    let guild = guild.to_guild_cached().chain_err(|| "could not find guild")?;
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
    let member = match members.choose(&mut thread_rng()) {
      Some(u) => u,
      None => return Err("Could not randomly choose a reaction.".into())
    };
    Ok(format!("I've randomly selected {}!", member.mention()).into())
  }
}
