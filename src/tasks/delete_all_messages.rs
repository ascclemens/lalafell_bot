use LalafellBot;
use tasks::{RunsTask, FromConfig};
use config::Task;
use discord::GetMessages;
use discord::model::ChannelId;
use chrono::prelude::*;
use chrono::Duration;

use xivdb::error::*;
use serde_json;

use std::sync::Arc;
use std::thread;

#[derive(Debug)]
pub struct DeleteAllMessagesTask {
  config: DeleteAllMessagesTaskConfig,
  next_sleep: i64
}

impl RunsTask for DeleteAllMessagesTask {
  fn start(mut self, s: Arc<LalafellBot>) {
    loop {
      thread::sleep(Duration::seconds(self.next_sleep).to_std().unwrap());
      info!(target: "delete_messages", "Delete messages task running");
      let channel = ChannelId(self.config.channel);
      let messages = match s.discord.get_messages(channel, GetMessages::MostRecent, None) {
        Ok(m) => m,
        Err(e) => {
          warn!(target: "delete_messages", "Could not get messages for channel {}: {}", channel, e);
          self.next_sleep = 30;
          continue;
        }
      };
      for message in messages {
        if self.config.except.contains(&message.id.0) {
          continue;
        }
        let timestamp: DateTime<UTC> = match message.timestamp.parse() {
          Ok(t) => t,
          Err(e) => {
            warn!(target: "delete_messages", "Could not parse message timestamp: {}", e);
            continue;
          }
        };
        if timestamp + Duration::seconds(self.config.after) > UTC::now() {
          continue;
        }
        if let Err(e) = s.discord.delete_message(message.channel_id, message.id) {
          warn!(target: "delete_messages", "Could not delete message {}: {}", message.id, e);
        }
      }
      self.next_sleep = 60;
      info!(target: "delete_messages", "Delete messages task done");
    }
  }
}

impl FromConfig for DeleteAllMessagesTask {
  fn from_config(task: &Task) -> Result<Self> {
    let val = match task.config {
      None => return Err("delete_all_messages task is missing configuration".into()),
      Some(ref val) => val
    };
    let config: DeleteAllMessagesTaskConfig = serde_json::from_value(val.clone()).chain_err(|| "could not parse delete_all_messages configuration")?;
    Ok(DeleteAllMessagesTask {
      config: config,
      next_sleep: Default::default()
    })
  }
}

#[derive(Debug, Deserialize)]
pub struct DeleteAllMessagesTaskConfig {
  pub channel: u64,
  pub after: i64,
  pub except: Vec<u64>
}
