use LalafellBot;
use tasks::RunsTask;
use discord::GetMessages;
use discord::model::{ChannelId, MessageId};
use chrono::prelude::*;
use chrono::Duration;

use std::sync::Arc;
use std::thread;

#[derive(Debug)]
pub struct DeleteAllMessagesTask {
  channel: ChannelId,
  after: i64,
  except: Vec<MessageId>,
  next_sleep: i64
}

impl DeleteAllMessagesTask {
  pub fn new(channel: ChannelId, after: i64, except: Vec<MessageId>) -> DeleteAllMessagesTask {
    DeleteAllMessagesTask {
      channel: channel,
      after: after,
      except: except,
      next_sleep: 0
    }
  }
}

impl RunsTask for DeleteAllMessagesTask {
  fn start(mut self, s: Arc<LalafellBot>) {
    loop {
      thread::sleep(Duration::seconds(self.next_sleep).to_std().unwrap());
      info!(target: "delete_messages", "Delete messages task running");
      let messages = match s.discord.get_messages(self.channel, GetMessages::MostRecent, None) {
        Ok(m) => m,
        Err(e) => {
          warn!(target: "delete_messages", "Could not get messages for channel {}: {}", self.channel, e);
          self.next_sleep = 30;
          continue;
        }
      };
      for message in messages {
        if self.except.contains(&message.id) {
          continue;
        }
        let timestamp: DateTime<UTC> = match message.timestamp.parse() {
          Ok(t) => t,
          Err(e) => {
            warn!(target: "delete_messages", "Could not parse message timestamp: {}", e);
            continue;
          }
        };
        if timestamp + Duration::seconds(self.after) > UTC::now() {
          continue;
        }
        if let Err(e) = s.discord.delete_message(self.channel, message.id) {
          warn!(target: "delete_messages", "Could not delete message {}: {}", message.id, e);
        }
      }
      self.next_sleep = 60;
      info!(target: "delete_messages", "Delete messages task done");
    }
  }
}
