use crate::bot::BotEnv;
use crate::database::models::EphemeralMessage;
use crate::error::*;
use crate::tasks::{RunsTask, Wait};

use chrono::{Utc, Duration};

use diesel;
use diesel::prelude::*;

use serenity::model::id::ChannelId;

use std::sync::Arc;
use std::thread;

#[derive(Debug, Default)]
pub struct EphemeralMessageTask {
  next_sleep: i64
}

impl RunsTask for EphemeralMessageTask {
  fn start(mut self, env: Arc<BotEnv>) {
    loop {
      thread::sleep(Duration::seconds(self.next_sleep).to_std().unwrap());
      if self.next_sleep == 0 {
        self.next_sleep = 1800;
      }
      let next_half_hour = (Utc::now() + Duration::minutes(30)).timestamp();
      info!("Checking for ephemeral messages");
      let res: Result<Vec<EphemeralMessage>> = crate::bot::with_connection(|c| {
        use crate::database::schema::ephemeral_messages::dsl;
        dsl::ephemeral_messages
          .filter(dsl::expires_on.le(next_half_hour))
          .load(c)
      }).chain_err(|| "could not load ephemeral messages");
      let mut msgs = match res {
        Ok(m) => m,
        Err(e) => {
          warn!("error loading ephemeral messages: {}", e);
          continue;
        }
      };

      if msgs.is_empty() {
        continue;
      }

      msgs.sort_by_key(|m| m.expires_on);

      let thread_env = Arc::clone(&env);
      ::std::thread::spawn(move || {
        for (wait, eph) in Wait::new(msgs.into_iter().map(|m| (m.expires_on, m))) {
          ::std::thread::sleep(Duration::seconds(wait).to_std().unwrap());

          let channel = ChannelId(*eph.channel_id);
          match channel.delete_message(thread_env.http(), *eph.message_id) {
            Ok(_) => {
              if let Err(e) = crate::bot::with_connection(|c| diesel::delete(&eph).execute(c)) {
                warn!("could not delete ephemeral message (id: {}) from database: {}", eph.id, e);
              }
            },
            Err(e) => warn!("could not delete ephemeral message (id: {}): {}", eph.id, e)
          }
        }
      });

      info!("Done checking for ephemeral messages");
    }
  }
}
