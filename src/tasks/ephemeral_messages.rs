use bot::BotEnv;
use database::models::EphemeralMessage;
use error::*;
use tasks::RunsTask;

use chrono::{Utc, Duration, TimeZone};

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
  fn start(mut self, _: Arc<BotEnv>) {
    loop {
      thread::sleep(Duration::seconds(self.next_sleep).to_std().unwrap());
      if self.next_sleep == 0 {
        self.next_sleep = 1800;
      }
      let now = Utc::now();
      let now_timestamp = now.timestamp();
      let next_half_hour = (now + Duration::minutes(30)).timestamp();
      info!("Checking for ephemeral messages");
      let res: Result<Vec<EphemeralMessage>> = ::bot::with_connection(|c| {
        use database::schema::ephemeral_messages::dsl;
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

      let (mut past, future): (Vec<EphemeralMessage>, Vec<EphemeralMessage>) = msgs.into_iter().partition(|x| x.expires_on <= now_timestamp);

      let mut wait_durations: Vec<i64> = future
        .windows(2)
        .map(|slice| slice[1].expires_on - slice[0].expires_on)
        .collect();

      let seconds = Utc.timestamp(future[0].expires_on, 0).signed_duration_since(now).num_seconds();
      wait_durations.insert(0, seconds);

      let mut wait_durations: Vec<i64> = ::std::iter::repeat(0).take(past.len()).chain(wait_durations).collect();

      debug_assert!(wait_durations.iter().all(|&x| x >= 0));

      past.extend(future);

      debug_assert!(wait_durations.len() == past.len());

      ::std::thread::spawn(move || {
        loop {
          if wait_durations.is_empty() {
            return;
          }
          ::std::thread::sleep(Duration::seconds(wait_durations.remove(0)).to_std().unwrap());
          let eph = past.remove(0);
          let channel = ChannelId(*eph.channel_id);
          match channel.delete_message(*eph.message_id) {
            Ok(_) => {
              if let Err(e) = ::bot::with_connection(|c| diesel::delete(&eph).execute(c)) {
                warn!("could not delete ephemeral message (id: {}) from database: {}", eph.id, e);
              }
            }
            Err(e) => warn!("could not delete ephemeral message (id: {}): {}", eph.id, e)
          }
        }
      });

      info!("Done checking for ephemeral messages");
    }
  }
}
