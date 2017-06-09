use LalafellBot;
use tasks::{RunsTask, FromConfig};
use config::Task;
use discord::GetMessages;
use discord::model::ChannelId;
use chrono::prelude::*;
use chrono::Duration;

use error::*;
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
      info!("Delete messages task running");
      let channel = ChannelId(self.config.channel);
      let messages = match s.discord.get_messages(channel, GetMessages::MostRecent, None) {
        Ok(m) => m,
        Err(e) => {
          warn!("Could not get messages for channel {}: {}", channel, e);
          self.next_sleep = 30;
          continue;
        }
      };
      let mut to_delete = Vec::new();
      for message in messages {
        if self.config.except.contains(&message.id.0) {
          continue;
        }
        let timestamp: DateTime<UTC> = match message.timestamp.parse() {
          Ok(t) => t,
          Err(e) => {
            warn!("Could not parse message timestamp: {}", e);
            continue;
          }
        };
        if timestamp + Duration::seconds(self.config.after) > UTC::now() {
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
      self.next_sleep = 60;
      info!("Delete messages task done");
    }
  }
}

impl FromConfig for DeleteAllMessagesTask {
  fn from_config(task: &Task) -> Result<Self> {
    let val = match task.config {
      None => bail!("delete_all_messages task is missing configuration"),
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
  #[serde(default)]
  pub except: Vec<u64>
}
