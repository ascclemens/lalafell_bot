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

impl TimeoutCheckTask {
  pub fn check_timeout(timeout: &Timeout, check_against: i64) {
    if timeout.ends() >= check_against {
      return;
    }
    let mut member = match GuildId(*timeout.server_id).member(*timeout.user_id) {
      Ok(m) => m,
      Err(e) => {
        warn!("could not get member for timeout check: {}", e);
        return;
      }
    };
    if let Err(e) = member.remove_role(*timeout.role_id) {
      warn!("could not remove timeout role from {}: {}", *timeout.user_id, e);
    }
    if let Err(e) = ::bot::with_connection(|c| ::diesel::delete(timeout).execute(c)) {
      warn!("could not delete timeout {}: {}", timeout.id, e);
    }
  }
}

impl RunsTask for TimeoutCheckTask {
  fn start(mut self, env: Arc<BotEnv>) {
    loop {
      thread::sleep(Duration::seconds(self.next_sleep).to_std().unwrap());
      let now = Utc::now().timestamp();
      let timeouts: Vec<Timeout> = match ::bot::with_connection(|c| ::database::schema::timeouts::dsl::timeouts.load(c)) {
        Ok(t) => t,
        Err(e) => {
          warn!("could not load timeouts: {}", e);
          continue;
        }
      };
      for timeout in timeouts {
        TimeoutCheckTask::check_timeout(&timeout, now);
      }
      self.next_sleep = env.config.read().timeouts.role_check_interval.unwrap_or(30);
    }
  }
}
