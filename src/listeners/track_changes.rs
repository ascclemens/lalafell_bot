use database::models::{Message as DbMessage, NewMessage, NewEdit};

use serenity::client::{Context, EventHandler};
use serenity::model::channel::Message;
use serenity::model::id::UserId;
use serenity::model::event::MessageUpdateEvent;

use diesel::prelude::*;

#[allow(dead_code)]
pub struct TrackChanges;

impl TrackChanges {
  fn user_id(&self) -> UserId {
    ::serenity::CACHE.read().user.id
  }
}

impl EventHandler for TrackChanges {
  fn message(&self, _: Context, message: Message) {
    if message.author.id == self.user_id() {
      return;
    }
    ::bot::CONNECTION.with(|c| {
      let new_message = NewMessage {
        message_id: message.id.into(),
        channel_id: message.channel_id.into(),
        content: message.content.to_owned()
      };
      let res = ::diesel::insert_into(::database::schema::messages::table)
        .values(&new_message)
        .execute(c);
      if let Err(e) = res {
        warn!("couldn't add message to database: {}", e);
      }
    });
  }

  fn message_update(&self, _: Context, update: MessageUpdateEvent) {
    ::bot::CONNECTION.with(|c| {
      use database::schema::messages::dsl;
      let message: Result<DbMessage, _> = dsl::messages
        .filter(dsl::message_id.eq(update.id.0.to_string()))
        .first(c);
      if let Ok(m) = message {
        let new_edit = NewEdit {
          message_id: m.id,
          content: update.content.unwrap_or_default()
        };
        let res = ::diesel::insert_into(::database::schema::edits::table)
          .values(&new_edit)
          .execute(c);
        if let Err(e) = res {
          warn!("couldn't add edit to database: {}", e);
        }
      }
    });
  }
}
