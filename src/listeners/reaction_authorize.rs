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
  result_wrap! {
    fn receive(_ctx: Context, r: &Reaction, added: bool) -> Result<()> {
      let channel = match r.channel_id.to_channel().chain_err(|| "could not get channel")? {
        Channel::Guild(c) => c.read().clone(),
        _ => return Ok(())
      };
      let reactions: Vec<DbReaction> = ::bot::with_connection(|c| {
        use database::schema::reactions::dsl;
        dsl::reactions
          .filter(dsl::channel_id.eq(r.channel_id.to_u64())
            .and(dsl::server_id.eq(channel.guild_id.to_u64()))
            .and(dsl::message_id.eq(r.message_id.to_u64()))
            .and(dsl::emoji.eq(r.emoji.to_string())))
          .load(c)
      }).chain_err(|| "could not load reactions")?;
      let guild = channel.guild_id.to_partial_guild().chain_err(|| "could not get guild")?;
      let mut member = guild.member(r.user_id).chain_err(|| "could not get member")?;
      for reac in reactions {
        if added {
          member.add_role(*reac.role_id).chain_err(|| "could not add role")?;
        } else {
          member.remove_role(*reac.role_id).chain_err(|| "could not remove role")?;
        }
      }
      Ok(())
    } |e| warn!("{}", e)
  }
}
