use crate::database::models::{ToU64, Timeout};

use chrono::prelude::*;

use diesel::prelude::*;

use lalafell::error::*;

use serenity::{
  client::{Context, EventHandler},
  model::channel::{Message, Channel, GuildChannel},
  prelude::RwLock,
};

use std::sync::Arc;

#[allow(dead_code)]
pub struct Timeouts;

impl EventHandler for Timeouts {
  result_wrap! {
    fn message(&self, ctx: Context, message: Message) -> Result<()> {
      let channel = match message.channel_id.to_channel(&ctx).chain_err(|| "could not get channel")? {
        Channel::Guild(c) => c,
        _ => return Ok(()),
      };
      let timeout = crate::bot::with_connection(|c| {
        use crate::database::schema::timeouts::dsl;
        dsl::timeouts
          .filter(dsl::user_id.eq(message.author.id.to_u64()).and(dsl::server_id.eq(channel.read().guild_id.to_u64())))
          .first(c)
      });

      let timeout: Timeout = match timeout {
        Ok(t) => t,
        _ => return Ok(()),
      };

      if timeout.ends() < Utc::now().timestamp() {
        if let Err(e) = crate::bot::with_connection(|c| ::diesel::delete(&timeout).execute(c)) {
          warn!("could not delete timeout: {}", e);
        }
        return Ok(());
      }

      if let Err(e) = message.delete(&ctx) {
        warn!("could not delete message {} in {}: {}", message.id.0, message.channel_id.0, e);
      }
      Ok(())
    } |e| warn!("{}", e)
  }

  result_wrap! {
    fn channel_create(&self, ctx: Context, channel: Arc<RwLock<GuildChannel>>) -> Result<()> {
      let guild_id = channel.read().guild_id;
      let guild = guild_id.to_guild_cached(&ctx).chain_err(|| "could not find guild")?;
      if let Err(e) = crate::commands::timeout::set_up_timeouts(&ctx, &guild.read()) {
        warn!("could not add timeout overwrite to {}: {}", channel.read().id.0, e);
      }
      Ok(())
    } |e| warn!("{}", e)
  }
}
