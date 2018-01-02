use bot::LalafellBot;
use listeners::ReceivesEvents;
use discord::model::{Event, Message, MessageId, UserId};
use database::models::{Message as DbMessage, NewMessage, NewEdit};

use diesel::prelude::*;

use std::sync::Arc;

#[allow(dead_code)]
pub struct TrackChanges {
  bot: Arc<LalafellBot>
}

impl TrackChanges {
  pub fn new(bot: Arc<LalafellBot>) -> Self {
    TrackChanges {
      bot
    }
  }

  fn get_user_id(&self) -> UserId {
    let opt_state = self.bot.state.read().unwrap();
    let state = opt_state.as_ref().unwrap();
    state.user().id
  }

  fn handle_message(&self, message: &Message) {
    if message.author.id == self.get_user_id() {
      return;
    }
    ::bot::CONNECTION.with(|c| {
      let new_message = NewMessage {
        message_id: message.id.into(),
        channel_id: message.channel_id.into(),
        content: message.content.to_owned()
      };
      let res = ::diesel::insert(&new_message)
        .into(::database::schema::messages::table)
        .execute(c);
      if let Err(e) = res {
        warn!("couldn't add message to database: {}", e);
      }
    });
  }

  fn handle_message_update(&self, id: MessageId, content: String) {
    ::bot::CONNECTION.with(|c| {
      use database::schema::messages::dsl;
      let message: Result<DbMessage, _> = dsl::messages
        .filter(dsl::message_id.eq(id.0.to_string()))
        .first(c);
      if let Ok(m) = message {
        let new_edit = NewEdit {
          message_id: m.id,
          content: content
        };
        let res = ::diesel::insert(&new_edit)
          .into(::database::schema::edits::table)
          .execute(c);
        if let Err(e) = res {
          warn!("couldn't add edit to database: {}", e);
        }
      }
    });
  }
}

impl ReceivesEvents for TrackChanges {
  fn receive(&self, event: &Event) {
    match *event {
      Event::MessageCreate(ref m) => self.handle_message(m),
      Event::MessageUpdate {
        ref id,
        ref content,
        ..
      } => {
        self.handle_message_update(*id, content.clone().unwrap_or_default());
      },
      _ => {}
    }
  }
}
