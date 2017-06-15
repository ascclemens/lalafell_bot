use bot::LalafellBot;
use tasks::RunsTask;
use chrono::prelude::*;
use chrono::Duration;

use discord::model::{ServerId, UserId, RoleId};

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
      let mut database = s.database.write().unwrap();
      let now = UTC::now().timestamp();
      database.timeouts.retain(|t| {
        if t.ends() >= now {
          return true;
        }
        if let Err(e) = s.discord.remove_user_from_role(ServerId(t.server_id), UserId(t.user_id), RoleId(t.role_id)) {
          warn!("could not remove timeout role from {}: {}", t.user_id, e);
        }
        return false;
      });
      self.next_sleep = s.config.bot.timeouts.role_check_interval.unwrap_or(30);
    }
  }
}
