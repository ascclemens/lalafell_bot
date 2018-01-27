use database::models::{ToU64, Reaction as DbReaction};

use diesel::prelude::*;

use serenity::client::{Context, EventHandler};
use serenity::model::channel::{Channel, Reaction};

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
        _ => return Ok(())
      };
      let emoji = r.emoji.to_string();
      let reactions: Vec<DbReaction> = ::bot::CONNECTION.with(|c| {
        use database::schema::reactions::dsl;
        dsl::reactions
          .filter(dsl::channel_id.eq(r.channel_id.to_u64())
            .and(dsl::server_id.eq(channel.guild_id.to_u64()))
            .and(dsl::message_id.eq(r.message_id.to_u64()))
            .and(dsl::emoji.eq(emoji)))
          .load(c)
          .chain_err(|| "could not load reactions")
      })?;
      let guild = channel.guild_id.get().chain_err(|| "could not get guild")?;
      let mut member = guild.member(r.user_id).chain_err(|| "could not get member")?;
      for reac in reactions {
        if added {
          member.add_role(*reac.role_id).chain_err(|| "could not add role")?;
        } else {
          member.remove_role(*reac.role_id).chain_err(|| "could not remove role")?;
        }
      }
      Ok(())
    };
    if let Err(e) = inner() {
      warn!("ReactionAuthorize error: {}", e);
    }
  }
}
