use database::models::{ToU64, TemporaryRole};
use error::*;

use diesel::prelude::*;

use serenity::client::{Context, EventHandler};
use serenity::model::channel::Message;

pub struct TemporaryRolesListener;

impl EventHandler for TemporaryRolesListener {
  result_wrap! {
    fn message(&self, _ctx: Context, message: Message) -> Result<()> {
      let guild_channel = match message.channel().and_then(|c| c.guild()) {
        Some(g) => g,
        None => return Ok(())
      };
      let channel_id = guild_channel.read().id;
      let guild_id = guild_channel.read().guild_id;
      let mut member = guild_id.member(&message.author).chain_err(|| "could not get member")?;

      let temps: Vec<TemporaryRole> = ::bot::with_connection(|c| {
        use database::schema::temporary_roles::dsl;

        dsl::temporary_roles
          .filter(dsl::user_id.eq(message.author.id.to_u64())
            .and(dsl::guild_id.eq(guild_id.to_u64()))
            .and(dsl::messages.is_not_null()))
          .load(c)
      }).chain_err(|| "could not get temporary roles")?;

      for mut temp in temps {
        if message.id == *temp.message_id {
          continue;
        }

        if let Some(chan_id) = temp.channel_id {
          if chan_id as u64 != channel_id.0 {
            continue;
          }
        }

        temp.messages = temp.messages.map(|x| x - 1);

        if temp.messages == Some(0) {
          member.remove_role(*temp.role_id).chain_err(|| "could not remove role")?;
          ::bot::with_connection(|c| {
            ::diesel::delete(&temp).execute(c)
          }).chain_err(|| "could not delete temporary role")?;
        } else {
          ::bot::with_connection(|c| {
            temp.save_changes::<TemporaryRole>(c)
          }).chain_err(|| "could not save temporary role changes")?;
        }
      }
      Ok(())
    } |e| warn!("{}", e)
  }
}