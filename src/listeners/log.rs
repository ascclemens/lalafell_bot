use database::models::{ToU64, LogChannel};

use diesel::prelude::*;

use serenity::prelude::Mentionable;
use serenity::client::{Context, EventHandler};
use serenity::model::id::{GuildId, ChannelId};
use serenity::model::user::User;
use serenity::model::guild::Member;

pub struct Log;

impl Log {
  fn get_log_channel<G: Into<GuildId>>(&self, guild: G) -> Option<ChannelId> {
    let log_channel: Option<LogChannel> = ::bot::CONNECTION.with(|c| {
      use database::schema::log_channels::dsl;
      dsl::log_channels.filter(dsl::server_id.eq(guild.into().to_u64()))
        .first(c)
        .ok()
    });
    log_channel.map(|x| ChannelId(*x.channel_id))
  }
}

impl EventHandler for Log {
  fn guild_member_removal(&self, _: Context, guild: GuildId, user: User, member: Option<Member>) {
    let channel_id = some_or!(self.get_log_channel(guild), return);
    let name = member.as_ref().map(|x| x.display_name().to_string()).unwrap_or_else(|| user.name.clone());
    let mention = member.as_ref().map(|x| x.mention()).unwrap_or_else(|| user.mention());
    channel_id.send_message(|m| m.content(&format!("{} ({}) has left the server.", mention, name))).ok();
  }

  fn guild_member_addition(&self, _: Context, guild: GuildId, member: Member) {
    let channel_id = some_or!(self.get_log_channel(guild), return);
    channel_id.send_message(|m| m.content(&format!("{} ({}) has joined the server.", member.mention(), member.display_name()))).ok();
  }
}
