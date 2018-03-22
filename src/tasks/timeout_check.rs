use bot::BotEnv;
use tasks::{RunsTask, Wait};
use chrono::prelude::*;
use chrono::Duration;
use database::models::Timeout;

use serenity::model::id::GuildId;

use diesel::prelude::*;

use std::sync::Arc;
use std::thread;

#[derive(Default)]
pub struct TimeoutCheckTask {
  ran_once: bool
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
      let sleep = if self.ran_once {
        env.config.read().timeouts.role_check_interval.unwrap_or(300)
      } else {
        self.ran_once = true;
        0
      };
      thread::sleep(Duration::seconds(sleep).to_std().unwrap());
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

      if timeouts.is_empty() {
        continue;
      }

      ::std::thread::spawn(move || {
        for (wait, timeout) in Wait::new(timeouts.into_iter().map(|t| (t.ends(), t))) {
          ::std::thread::sleep(Duration::seconds(wait).to_std().unwrap());
          remove_timeout(&timeout);
        }
      });
    }
  }
}
