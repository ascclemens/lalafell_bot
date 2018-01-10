use bot::BotEnv;

use serenity::prelude::Mentionable;
use serenity::client::{Context, EventHandler};
use serenity::model::id::{GuildId, ChannelId};
use serenity::model::user::User;
use serenity::model::guild::Member;

use std::sync::Arc;

pub struct Log {
  env: Arc<BotEnv>
}

impl Log {
  pub fn new(env: Arc<BotEnv>) -> Self {
    Log { env }
  }
}

impl EventHandler for Log {
  fn guild_member_removal(&self, _: Context, guild: GuildId, user: User, member: Option<Member>) {
    let channel_id = match self.env.config.read().bot.log.get(&guild.0) {
      Some(c) => ChannelId(*c),
      None => return
    };
    let name = member.as_ref().map(|x| x.display_name().to_string()).unwrap_or_else(|| user.name.clone());
    let mention = member.as_ref().map(|x| x.mention()).unwrap_or_else(|| user.mention());
    channel_id.send_message(|m| m.content(&format!("{} ({}) has left the server.", mention, name))).ok();
  }

  fn guild_member_addition(&self, _: Context, guild: GuildId, member: Member) {
    let channel_id = match self.env.config.read().bot.log.get(&guild.0) {
      Some(c) => ChannelId(*c),
      None => return
    };
    channel_id.send_message(|m| m.content(&format!("{} ({}) has joined the server.", member.mention(), member.display_name()))).ok();
  }
}
