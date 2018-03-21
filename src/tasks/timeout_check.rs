use bot::BotEnv;
use tasks::{RunsTask, Wait};
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

pub fn remove_timeout(timeout: &Timeout) {
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

impl RunsTask for TimeoutCheckTask {
  fn start(mut self, env: Arc<BotEnv>) {
    loop {
      thread::sleep(Duration::seconds(self.next_sleep).to_std().unwrap());
      let now = Utc::now();
      let next_five_minutes = (now + Duration::minutes(5)).timestamp();
      let mut timeouts: Vec<Timeout> = match ::bot::with_connection(|c| ::database::schema::timeouts::dsl::timeouts.load(c)) {
        Ok(t) => t,
        Err(e) => {
          warn!("could not load timeouts: {}", e);
          continue;
        }
      };
      timeouts.retain(|t| t.ends() <= next_five_minutes);
      ::std::thread::spawn(move || {
        for (wait, timeout) in Wait::new(timeouts.into_iter().map(|t| (t.ends(), t))) {
          ::std::thread::sleep(Duration::seconds(wait).to_std().unwrap());
          remove_timeout(&timeout);
        }
      });
      self.next_sleep = env.config.read().timeouts.role_check_interval.unwrap_or(300);
    }
  }
}
