use bot::BotEnv;
use tasks::RunsTask;
use chrono::prelude::*;
use chrono::Duration;
use database::models::Timeout;

use serenity::model::id::GuildId;

use diesel::prelude::*;

use std::sync::Arc;
use std::thread;

pub struct TimeoutCheckTask {
  next_sleep: i64
}

impl TimeoutCheckTask {
  pub fn new() -> TimeoutCheckTask {
    TimeoutCheckTask {
      next_sleep: 0
    }
  }
}

impl RunsTask for TimeoutCheckTask {
  fn start(mut self, env: Arc<BotEnv>) {
    loop {
      thread::sleep(Duration::seconds(self.next_sleep).to_std().unwrap());
      let now = Utc::now().timestamp();
      let timeouts: Vec<Timeout> = match ::bot::CONNECTION.with(|c| ::database::schema::timeouts::dsl::timeouts.load(c)) {
        Ok(t) => t,
        Err(e) => {
          warn!("could not load timeouts: {}", e);
          continue;
        }
      };
      for timeout in timeouts {
        if timeout.ends() >= now {
          continue;
        }
        let mut member = match GuildId(*timeout.server_id).member(*timeout.user_id) {
          Ok(m) => m,
          Err(e) => {
            warn!("could not get member for timeout check: {}", e);
            continue;
          }
        };
        if let Err(e) = member.remove_role(*timeout.role_id) {
          warn!("could not remove timeout role from {}: {}", *timeout.user_id, e);
        }
        ::bot::CONNECTION.with(|c| {
          if let Err(e) = ::diesel::delete(&timeout).execute(c) {
            warn!("could not delete timeout {}: {}", timeout.id, e);
          }
        });
      }
      self.next_sleep = env.config.bot.timeouts.role_check_interval.unwrap_or(30);
    }
  }
}
