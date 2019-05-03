use crate::{
  bot::BotEnv,
  tasks::{RunsTask, Wait},
  database::models::Timeout,
};

use chrono::{
  Duration,
  prelude::*,
};

use diesel::prelude::*;

use std::{
  sync::Arc,
  thread,
};

#[derive(Default)]
pub struct TimeoutCheckTask {
  ran_once: bool,
}

pub fn remove_timeout(env: &BotEnv, timeout: &Timeout) {
  // FIXME: this may not work if the character is not in the cache, but this is needed to compile
  let mut member = match env.cache().read().member(*timeout.server_id, *timeout.user_id) {
    Some(m) => m,
    None => {
      warn!("could not get member for timeout check: missing in cache");
      return;
    },
  };
  if let Err(e) = member.remove_role(env.http(), *timeout.role_id) {
    warn!("could not remove timeout role from {}: {}", *timeout.user_id, e);
  }
  if let Err(e) = crate::bot::with_connection(|c| diesel::delete(timeout).execute(c)) {
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
      let mut timeouts: Vec<Timeout> = match crate::bot::with_connection(|c| crate::database::schema::timeouts::dsl::timeouts.load(c)) {
        Ok(t) => t,
        Err(e) => {
          warn!("could not load timeouts: {}", e);
          continue;
        },
      };
      timeouts.retain(|t| t.ends() <= next_five_minutes);

      if timeouts.is_empty() {
        continue;
      }

      let thread_env = Arc::clone(&env);
      std::thread::spawn(move || {
        for (wait, timeout) in Wait::new(timeouts.into_iter().map(|t| (t.ends(), t))) {
          std::thread::sleep(Duration::seconds(wait).to_std().unwrap());
          remove_timeout(&thread_env, &timeout);
        }
      });
    }
  }
}
