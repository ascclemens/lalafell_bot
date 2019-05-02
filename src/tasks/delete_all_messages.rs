use bot::BotEnv;
use tasks::RunsTask;
use database::models::DeleteAllMessages;

use serenity::model::id::ChannelId;

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
  fn start(mut self, env: Arc<BotEnv>) {
    loop {
      thread::sleep(Duration::seconds(self.next_sleep).to_std().unwrap());
      info!("Delete messages task running");
      let dams: Result<Vec<DeleteAllMessages>> = ::bot::with_connection(|c| {
        use database::schema::delete_all_messages::dsl;
        dsl::delete_all_messages.load(c)
      }).chain_err(|| "could not load delete_all_messages");
      let dams = match dams {
        Ok(d) => d,
        Err(e) => {
          warn!("could not load delete_all_messages: {}", e);
          continue;
        }
      };
      for dam in dams {
        let channel = ChannelId(*dam.channel_id);
        let messages = match channel.messages(env.http(), |m| m) {
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
          if message.timestamp.with_timezone(&Utc) + Duration::seconds(i64::from(dam.after)) > Utc::now() {
            continue;
          }
          to_delete.push(message);
        }
        if !to_delete.is_empty() {
          info!("{} message{} to delete", to_delete.len(), if to_delete.len() == 1 { "" } else { "s" });
        }
        for chunk in to_delete.chunks(100) {
          info!("Deleting chunk of {} message{}", chunk.len(), if chunk.len() == 1 { "" } else { "s" });
          let result = if chunk.len() == 1 {
            channel.delete_message(env.http(), chunk[0].id)
          } else {
            let ids: Vec<_> = chunk.iter().map(|m| m.id).collect();
            channel.delete_messages(env.http(), ids)
          };
          if let Err(e) = result {
            warn!("Could not delete messages: {}", e);
          }
        }
      }
      info!("Delete messages task done");
      self.next_sleep = 60;
    }
  }
}
