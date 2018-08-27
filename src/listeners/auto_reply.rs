use database::models::{ToU64, AutoReply};
use filters::Filter;
use error::*;

use diesel::prelude::*;

use serenity::client::{Context, EventHandler};
use serenity::model::id::{ChannelId, GuildId, UserId};
use serenity::model::channel::{Channel, Message};
use serenity::model::guild::{Role, Member};
use serenity::model::misc::Mentionable;

use chrono::Utc;

use std::sync::Mutex;
use std::collections::HashMap;

#[derive(Default)]
pub struct AutoReplyListener {
  last_sends: Mutex<HashMap<(UserId, i32), i64>>
}

enum UserIdOrMember {
  UserId(UserId),
  Member(Member)
}

impl EventHandler for AutoReplyListener {
  result_wrap! {
    fn guild_member_addition(&self, _ctx: Context, guild: GuildId, member: Member) -> Result<()> {
      let replies: Vec<AutoReply> = ::bot::with_connection(|c| {
        use database::schema::auto_replies::dsl;
        dsl::auto_replies
          .filter(dsl::server_id.eq(guild.to_u64())
            .and(dsl::on_join.eq(true)))
          .load(c)
      }).chain_err(|| "could not load auto_replies")?;
      let user = UserIdOrMember::Member(member.clone());
      self.receive(replies, user, guild)
    } |e| warn!("{}", e)
  }

  result_wrap! {
    fn message(&self, _ctx: Context, m: Message) -> Result<()> {
      if m.author.id == ::serenity::CACHE.read().user.id {
        return Ok(());
      }
      let replies: Vec<AutoReply> = ::bot::with_connection(|c| {
        use database::schema::auto_replies::dsl;
        dsl::auto_replies
          .filter(dsl::channel_id.eq(m.channel_id.to_u64())
            .and(dsl::on_join.eq(false)))
          .load(c)
      }).chain_err(|| "could not load auto_replies")?;
      let user = UserIdOrMember::UserId(m.author.id);
      let guild = match m.channel_id.to_channel() {
        Ok(Channel::Guild(c)) => c.read().guild_id,
        Ok(_) => bail!("wrong type of channel for auto reply"),
        Err(e) => bail!("could not get channel for auto reply: {}", e)
      };
      self.receive(replies, user, guild)
    } |e| warn!("{}", e)
  }
}

impl AutoReplyListener {
  fn receive(&self, replies: Vec<AutoReply>, user: UserIdOrMember, guild: GuildId) -> Result<()> {
    let live_server = match guild.to_guild_cached() {
      Some(g) => g.read().clone(),
      None => bail!("could not find guild")
    };
    let member = match user {
      UserIdOrMember::Member(m) => m,
      UserIdOrMember::UserId(u) => match live_server.members.iter().find(|&(id, _)| *id == u) {
        Some((_, m)) => m.clone(),
        None => bail!("could not find member for auto reply")
      }
    };
    let user_id = member.user.read().id;
    let roles: Vec<Role> = live_server.roles.values().cloned().collect();
    let mut last_sends = self.last_sends.lock().unwrap();
    for reply in replies {
      if let Some(ref filters_string) = reply.filters {
        match Filter::all_filters(filters_string) {
          Some(filters) => if !filters.iter().all(|f| f.matches(&member, &roles)) {
            continue;
          },
          None => warn!("invalid filters: `{}`", filters_string)
        }
      }
      let last_send = last_sends.entry((user_id, reply.id)).or_insert(0);
      if *last_send + i64::from(reply.delay) >= Utc::now().timestamp() {
        continue;
      }
      ChannelId(*reply.channel_id).send_message(|c| c.embed(|e| e.description(&reply.message.replace("{mention}", &member.mention())))).ok();
      *last_send = Utc::now().timestamp();
    }
    Ok(())
  }
}
