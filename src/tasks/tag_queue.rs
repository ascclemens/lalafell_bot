use tasks::RunsTask;
use bot::BotEnv;
use commands::tag::Tagger;
use database::models::TagQueue;

use chrono::Duration;

use diesel::prelude::*;

use std::sync::Arc;
use std::thread;

pub struct TagQueueTask {
  pub next_sleep: i64
}

impl Default for TagQueueTask {
  fn default() -> Self {
    TagQueueTask { next_sleep: 30 }
  }
}

impl RunsTask for TagQueueTask {
  fn start(mut self, env: Arc<BotEnv>) {
    loop {
      thread::sleep(Duration::seconds(self.next_sleep).to_std().unwrap());
      self.next_sleep = Duration::minutes(30).num_seconds();
      info!("Checking tag queue");
      let mut queue: Vec<TagQueue> = match ::bot::CONNECTION.with(|c| {
        use database::schema::tag_queue::dsl;
        dsl::tag_queue.load(c)
      }) {
        Ok(t) => t,
        Err(e) => {
          warn!("could not load tag queue: {}", e);
          continue;
        }
      };
      let len = queue.len();
      if len == 0 {
        info!("No queued tags");
        continue;
      } else {
        info!("{} queued tag{}", len, if len == 1 { "" } else { "s" });
      }
      queue.retain(|item| {
        match Tagger::search_tag(
          env.as_ref(),
          (*item.user_id).into(),
          (*item.server_id).into(),
          &item.server,
          &item.character,
          false
        ) {
          Ok(None) => true,
          _ => false
        }
      });
      info!("Successfully tagged {}/{} queued tags", queue.len(), len);
      for remove in queue {
        if let Err(e) = ::bot::CONNECTION.with(|c| ::diesel::delete(&remove).execute(c)) {
          warn!("could not remove item from queue after successful tagging: {}", e);
        }
      }
    }
  }
}
