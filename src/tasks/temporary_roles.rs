use crate::{
  bot::BotEnv,
  tasks::{RunsTask, Wait},
  database::models::TemporaryRole,
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
pub struct TemporaryRolesTask {
  next_sleep: i64,
}

pub fn remove_temporary_role(env: &BotEnv, temp: &TemporaryRole) {
  let mut member = match env.cache().read().member(*temp.guild_id, *temp.user_id) {
    Some(m) => m,
    None => {
      warn!("could not get member for temp role removal: missing in cache");
      return;
    },
  };
  if let Err(e) = member.remove_role(env.http(), *temp.role_id) {
    warn!("could not remove temp role {}: {}", temp.id, e);
  }
  if let Err(e) = crate::bot::with_connection(|c| ::diesel::delete(temp).execute(c)) {
    warn!("could not delete temp role {} from database: {}", temp.id, e);
  }
}

impl RunsTask for TemporaryRolesTask {
  fn start(mut self, env: Arc<BotEnv>) {
    loop {
      thread::sleep(Duration::seconds(self.next_sleep).to_std().unwrap());
      if self.next_sleep == 0 {
        self.next_sleep = 600;
      }
      let now = Utc::now();
      let next_ten_minutes = (now + Duration::minutes(10)).timestamp();
      let temp_roles: Vec<TemporaryRole> = match crate::bot::with_connection(|c| {
        use crate::database::schema::temporary_roles::dsl;
        dsl::temporary_roles.filter(dsl::expires_on.le(next_ten_minutes)).load(c)
      }) {
        Ok(t) => t,
        Err(e) => {
          warn!("could not load temporary roles: {}", e);
          continue;
        },
      };

      if temp_roles.is_empty() {
        continue;
      }

      let thread_env = Arc::clone(&env);
      std::thread::spawn(move || {
        for (wait, temp_role) in Wait::new(temp_roles.into_iter().map(|t| (t.expires_on.unwrap(), t))) {
          std::thread::sleep(Duration::seconds(wait).to_std().unwrap());
          remove_temporary_role(&thread_env, &temp_role);
        }
      });
    }
  }
}
