use bot::LalafellBot;
use tasks::RunsTask;
use database::models::DeleteAllMessages;

use discord::GetMessages;
use discord::model::ChannelId;

use chrono::prelude::*;
use chrono::Duration;

use error::*;

use diesel::prelude::*;

use std::sync::Arc;
use std::thread;

#[derive(Debug)]
pub struct DeleteAllMessagesTask {
  next_sleep: i64
}

impl DeleteAllMessagesTask {
  pub fn new() -> Self {
    DeleteAllMessagesTask {
      next_sleep: 30
    }
  }
}

impl RunsTask for DeleteAllMessagesTask {
  fn start(mut self, s: Arc<LalafellBot>) {
    loop {
      thread::sleep(Duration::seconds(self.next_sleep).to_std().unwrap());
      info!("Delete messages task running");
      let dams: ::std::result::Result<Vec<DeleteAllMessages>, _> = ::bot::CONNECTION.with(|c| {
        use database::schema::delete_all_messages::dsl;
        dsl::delete_all_messages.load(c).chain_err(|| "could not load delete_all_messages")
      });
      let dams = match dams {
        Ok(d) => d,
        Err(e) => {
          warn!("could not load delete_all_messages: {}", e);
          continue;
        }
      };
      for dam in dams {
        let channel = ChannelId(*dam.channel_id);
        let messages = match s.discord.get_messages(channel, GetMessages::MostRecent, None) {
          Ok(m) => m,
          Err(e) => {
            warn!("Could not get messages for channel {}: {}", channel, e);
            continue;
          }
        };
        let mut to_delete = Vec::new();
        let exclude = dam.exclude();
        for message in messages {
          if exclude.contains(&message.id.0) {
            continue;
          }
          if message.timestamp.with_timezone(&Utc) + Duration::seconds(dam.after as i64) > Utc::now() {
            continue;
          }
          to_delete.push(message);
        }
        for chunk in to_delete.chunks(100) {
          let result = if chunk.len() == 1 {
            s.discord.delete_message(channel, chunk[0].id)
          } else {
            let ids: Vec<_> = chunk.iter().map(|m| m.id).collect();
            s.discord.delete_messages(channel, &ids)
          };
          if let Err(e) = result {
            warn!("Could not delete messages: {}", e);
          }
        }
        info!("Delete messages task done");
      }
      self.next_sleep = 60;
    }
  }
}
