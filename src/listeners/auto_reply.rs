use bot::LalafellBot;
use discord::model::{Event, ChannelId, UserId, Channel, ServerId, Member};
use database::models::AutoReply;
use filters::Filter;
use error::*;

use lalafell::listeners::ReceivesEvents;

use diesel::prelude::*;

use chrono::Utc;

use std::sync::{Arc, Mutex};
use std::collections::HashMap;

pub struct AutoReplyListener {
  bot: Arc<LalafellBot>,
  last_sends: Mutex<HashMap<UserId, i64>>
}

impl AutoReplyListener {
  pub fn new(bot: Arc<LalafellBot>) -> AutoReplyListener {
    AutoReplyListener {
      bot: bot,
      last_sends: Mutex::default()
    }
  }
}

enum UserIdOrMember {
  UserId(UserId),
  Member(Member)
}

impl ReceivesEvents for AutoReplyListener {
  fn receive(&self, event: &Event) {
    let current_user = try_or!(self.bot.discord.get_current_user(), return);
    let replies: Option<(Result<Vec<AutoReply>>, UserIdOrMember, ServerId)> = ::bot::CONNECTION.with(|c| {
      use database::schema::auto_replies::dsl;
      match *event {
        Event::ServerMemberAdd(ref server_id, ref member) if member.user.id != current_user.id => {
          Some((
            dsl::auto_replies
              .filter(dsl::server_id.eq(server_id.0.to_string())
                .and(dsl::on_join.eq(true)))
              .load(c)
              .chain_err(|| "could not load auto_replies"),
            UserIdOrMember::Member(member.clone()),
            *server_id
          ))
        }
        Event::MessageCreate(ref m) if m.author.id != current_user.id => {
          Some((
            dsl::auto_replies
              .filter(dsl::channel_id.eq(m.channel_id.0.to_string())
                .and(dsl::on_join.eq(false)))
              .load(c)
              .chain_err(|| "could not load auto_replies"),
            UserIdOrMember::UserId(m.author.id),
            match self.bot.discord.get_channel(m.channel_id) {
              Ok(Channel::Public(c)) => c.server_id,
              Ok(_) => {
                warn!("wrong type of channel for auto reply");
                return None;
              }
              Err(e) => {
                warn!("could not get channel for auto reply: {}", e);
                return None;
              }
            }
          ))
        }
        _ => None
      }
    });
    let (replies, user, server) = some_or!(replies, return);
    let live_server = {
      let state_opt = self.bot.state.read().unwrap();
      let state = state_opt.as_ref().unwrap();
      let s = state.servers().iter().find(|s| s.id == server);
      match s {
        Some(s) => s.clone(),
        None => {
          warn!("could not find server for auto reply: {}", server);
          return;
        }
      }
    };
    let member = match user {
      UserIdOrMember::Member(m) => m,
      UserIdOrMember::UserId(u) => match live_server.members.iter().find(|m| m.user.id == u) {
        Some(m) => m.clone(),
        None => {
          warn!("could not find member for auto reply");
          return;
        }
      }
    };
    let roles = live_server.roles;
    let mut last_sends = self.last_sends.lock().unwrap();
    for reply in try_or!(replies, return) {
      if let Some(ref filters_string) = reply.filters {
        match filters_string.split(' ').map(Filter::parse).collect::<Option<Vec<_>>>() {
          Some(filters) => if !filters.iter().all(|f| f.matches(&member, &roles)) {
            continue;
          },
          None => warn!("invalid filters: `{}`", filters_string)
        }
      }
      let mut last_send = last_sends.entry(member.user.id).or_insert(0);
      if *last_send + reply.delay as i64 >= Utc::now().timestamp() {
        continue;
      }
      self.bot.discord.send_embed(ChannelId(*reply.channel_id), "", |e| e.description(&reply.message.replace("{mention}", &member.user.mention().to_string()))).ok();
      *last_send = Utc::now().timestamp()
    }
  }
}
