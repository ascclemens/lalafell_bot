use bot::LalafellBot;
use discord::model::{Event, ChannelId};
use database::models::AutoReply;
use error::*;

use lalafell::listeners::ReceivesEvents;

use diesel::prelude::*;

use std::sync::Arc;

pub struct AutoReplyListener {
  bot: Arc<LalafellBot>
}

impl AutoReplyListener {
  pub fn new(bot: Arc<LalafellBot>) -> AutoReplyListener {
    AutoReplyListener {
      bot: bot
    }
  }
}

impl ReceivesEvents for AutoReplyListener {
  fn receive(&self, event: &Event) {
    let replies: Vec<AutoReply> = match *event {
      Event::ServerMemberAdd(ref server_id, _) => {
        try_or!(::bot::CONNECTION.with(|c| {
          use database::schema::auto_replies::dsl;
          dsl::auto_replies
            .filter(dsl::server_id.eq(server_id.0.to_string())
              .and(dsl::on_join.eq(true)))
            .load(c)
            .chain_err(|| "could not load auto_replies")
        }), return)
      }
      Event::MessageCreate(ref m) => {
        try_or!(::bot::CONNECTION.with(|c| {
          use database::schema::auto_replies::dsl;
          dsl::auto_replies
            .filter(dsl::channel_id.eq(m.channel_id.0.to_string())
              .and(dsl::on_join.eq(false)))
            .load(c)
            .chain_err(|| "could not load auto_replies")
        }), return)
      }
      _ => return
    };
    for reply in replies {
      self.bot.discord.send_embed(ChannelId(*reply.channel_id), "", |e| e.description(&reply.message)).ok();
    }
  }
}
