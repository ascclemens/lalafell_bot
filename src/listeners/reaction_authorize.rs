use database::models::Reaction as DbReaction;

use diesel::prelude::*;

use serenity::client::{Context, EventHandler};
use serenity::model::channel::{Channel, Reaction, ReactionType};

use error::*;

pub struct ReactionAuthorize;

impl EventHandler for ReactionAuthorize {
  fn reaction_add(&self, context: Context, reaction: Reaction) {
    ReactionAuthorize::receive(context, &reaction, true);
  }

  fn reaction_remove(&self, context: Context, reaction: Reaction) {
    ReactionAuthorize::receive(context, &reaction, false);
  }
}

impl ReactionAuthorize {
  fn receive(_: Context, r: &Reaction, added: bool) {
    let inner = || -> Result<()> {
      let channel = match r.channel_id.get().chain_err(|| "could not get channel")? {
        Channel::Guild(c) => c.read().clone(),
        _ => bail!("invalid channel: {}", r.channel_id.0)
      };
      let emoji = match r.emoji {
        ReactionType::Unicode(ref emoji)  => emoji,
        _ => return Ok(())
      };
      let reactions: Vec<DbReaction> = ::bot::CONNECTION.with(|c| {
        use database::schema::reactions::dsl;
        dsl::reactions
          .filter(dsl::channel_id.eq(r.channel_id.0.to_string())
            .and(dsl::server_id.eq(channel.guild_id.0.to_string()))
            .and(dsl::message_id.eq(r.message_id.0.to_string()))
            .and(dsl::emoji.eq(emoji)))
          .load(c)
          .chain_err(|| "could not load reactions")
      })?;
      let guild = channel.guild_id.get().chain_err(|| "could not get guild")?;
      let mut member = guild.member(r.user_id).chain_err(|| "could not get member")?;
      for reac in reactions {
        let role = match guild.role_by_name(&reac.role) {
          Some(r) => r,
          None => {
            warn!("couldn't find role for name \"{}\"", reac.role);
            continue;
          }
        };
        if added {
          member.add_role(role.id).chain_err(|| "could not add role")?;
        } else {
          member.remove_role(role.id).chain_err(|| "could not remove role")?;
        }
      }
      Ok(())
    };
    if let Err(e) = inner() {
      warn!("ReactionAuthorize error: {}", e);
    }
  }
}
