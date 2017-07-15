use bot::LalafellBot;
use tasks::RunsTask;
use chrono::prelude::*;
use chrono::Duration;
use database::models::Timeout;

use discord::model::{ServerId, UserId, RoleId};

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
  fn start(mut self, s: Arc<LalafellBot>) {
    loop {
      thread::sleep(Duration::seconds(self.next_sleep).to_std().unwrap());
      let now = Utc::now().timestamp();
      let timeouts: Vec<Timeout> = match ::bot::CONNECTION.with(|c| ::database::schema::timeouts::dsl::timeouts.load(c)) {
        Ok(t) => t,
        Err(e) => {
          warn!("could not load timeouts: {}", e);
          return;
        }
      };
      for timeout in timeouts {
        if timeout.ends() >= now {
          continue;
        }
        if let Err(e) = s.discord.remove_user_from_role(ServerId(*timeout.server_id), UserId(*timeout.user_id), RoleId(*timeout.role_id)) {
          warn!("could not remove timeout role from {}: {}", *timeout.user_id, e);
        }
        ::bot::CONNECTION.with(|c| {
          if let Err(e) = ::diesel::delete(&timeout).execute(c) {
            warn!("could not delete timeout {}: {}", timeout.id, e);
          }
        });
      }
      self.next_sleep = s.config.bot.timeouts.role_check_interval.unwrap_or(30);
    }
  }
}
