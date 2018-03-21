use bot::BotEnv;
use tasks::{RunsTask, Wait};
use chrono::prelude::*;
use chrono::Duration;
use database::models::TemporaryRole;

use serenity::model::id::GuildId;

use diesel::prelude::*;

use std::sync::Arc;
use std::thread;

#[derive(Default)]
pub struct TemporaryRolesTask {
  next_sleep: i64
}

pub fn remove_temporary_role(temp: &TemporaryRole) {
  let mut member = match GuildId(*temp.guild_id).member(*temp.user_id) {
    Ok(m) => m,
    Err(e) => {
      warn!("could not get member for temp role removal: {}", e);
      return;
    }
  };
  if let Err(e) = member.remove_role(*temp.role_id) {
    warn!("could not remove temp role {}: {}", temp.id, e);
  }
  if let Err(e) = ::bot::with_connection(|c| ::diesel::delete(temp).execute(c)) {
    warn!("could not delete temp role {} from database: {}", temp.id, e);
  }
}

impl RunsTask for TemporaryRolesTask {
  fn start(mut self, _: Arc<BotEnv>) {
    loop {
      thread::sleep(Duration::seconds(self.next_sleep).to_std().unwrap());
      if self.next_sleep == 0 {
        self.next_sleep = 600;
      }
      let now = Utc::now();
      let next_ten_minutes = (now + Duration::minutes(10)).timestamp();
      let temp_roles: Vec<TemporaryRole> = match ::bot::with_connection(|c| {
        use database::schema::temporary_roles::dsl;
        dsl::temporary_roles.filter(dsl::expires_on.le(next_ten_minutes)).load(c)
      }) {
        Ok(t) => t,
        Err(e) => {
          warn!("could not load temporary roles: {}", e);
          continue;
        }
      };
      ::std::thread::spawn(move || {
        for (wait, temp_role) in Wait::new(temp_roles.into_iter().map(|t| (t.expires_on.unwrap(), t))) {
          ::std::thread::sleep(Duration::seconds(wait).to_std().unwrap());
          remove_temporary_role(&temp_role);
        }
      });
    }
  }
}
