use database::models::{ToU64, Timeout};
use lalafell::error::*;

use serenity::prelude::RwLock;
use serenity::client::{Context, EventHandler};
use serenity::model::channel::{Message, Channel, GuildChannel};

use diesel::prelude::*;

use chrono::prelude::*;

use std::sync::Arc;

#[allow(dead_code)]
pub struct Timeouts;

impl EventHandler for Timeouts {
  result_wrap! {
    fn message(&self, _ctx: Context, message: Message) -> Result<()> {
      let channel = match message.channel_id.get().chain_err(|| "could not get channel")? {
        Channel::Guild(c) => c,
        _ => return Ok(())
      };
      let timeout = ::bot::CONNECTION.with(|c| {
        use database::schema::timeouts::dsl;
        dsl::timeouts
          .filter(dsl::user_id.eq(message.author.id.to_u64()).and(dsl::server_id.eq(channel.read().guild_id.to_u64())))
          .first(c)
      });

      let timeout: Timeout = match timeout {
        Ok(t) => t,
        _ => return Ok(())
      };

      if timeout.ends() < Utc::now().timestamp() {
        ::bot::CONNECTION.with(|c| {
          if let Err(e) = ::diesel::delete(&timeout).execute(c).chain_err(|| "could not delete timeout") {
            warn!("could not delete timeout: {}", e);
          }
        });
        return Ok(());
      }

      if let Err(e) = message.delete() {
        warn!("could not delete message {} in {}: {}", message.id.0, message.channel_id.0, e);
      }
      Ok(())
    } |e| warn!("{}", e)
  }

  result_wrap! {
    fn channel_create(&self, _ctx: Context, channel: Arc<RwLock<GuildChannel>>) -> Result<()> {
      let guild_id = channel.read().guild_id;
      let guild = guild_id.find().chain_err(|| "could not find guild")?;
      if let Err(e) = ::commands::timeout::set_up_timeouts(&guild.read()) {
        warn!("could not add timeout overwrite to {}: {}", channel.read().id.0, e);
      }
      Ok(())
    } |e| warn!("{}", e)
  }
}
