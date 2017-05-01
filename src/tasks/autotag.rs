use LalafellBot;
use tasks::RunsTask;
use commands::tag::Tagger;
use discord::model::UserId;
use chrono::prelude::*;
use chrono::Duration;

use std::sync::Arc;
use std::thread;

pub struct AutoTagTask {
  pub next_sleep: i64
}

impl AutoTagTask {
  pub fn new() -> AutoTagTask {
    AutoTagTask {
      next_sleep: 30
    }
  }
}

impl RunsTask for AutoTagTask {
  fn start(mut self, s: Arc<LalafellBot>) {
    info!(target: "autotag", "Autotag task waiting {} seconds", self.next_sleep);
    loop {
      thread::sleep(Duration::seconds(self.next_sleep).to_std().unwrap());
      info!(target: "autotag", "Autotag task running");
      let time_to_update = {
        let database = s.database.lock().unwrap();
        database.autotags.last_updated + Duration::days(1).num_seconds() < UTC::now().timestamp()
      };
      if !time_to_update {
        info!(target: "autotag", "Not yet time to update, sleeping 30 minutes");
        self.next_sleep = Duration::minutes(30).num_seconds();
        continue;
      }
      info!(target: "autotag", "Time to update autotags");
      let users: Vec<(u64, u64, u64)> = {
        let database = s.database.lock().unwrap();
        database.autotags.users.iter().map(|u| (u.user_id, u.server_id, u.character_id)).collect()
      };
      {
        let option_state = s.state.lock().unwrap();
        let state = match option_state.as_ref() {
          Some(st) => st,
          None => {
            info!(target: "autotag", "Bot not connected. Will try again in 30 seconds.");
            self.next_sleep = 30;
            continue;
          }
        };
        for (user_id, server_id, character_id) in users {
          let server = match state.servers().iter().find(|s| s.id.0 == server_id) {
            Some(ser) => ser,
            None => {
              info!(target: "autotag", "Couldn't find server for user ID {}", user_id);
              continue;
            }
          };
          if let Err(e) = Tagger::tag(s.clone(), UserId(user_id), server, character_id) {
            info!(target: "autotag", "Couldn't update tag for user ID {}: {}", user_id, e);
          }
        }
      }
      {
        let mut database = s.database.lock().unwrap();
        database.autotags.last_updated = UTC::now().timestamp();
      };
      info!(target: "autotag", "Done updating autotags");
    }
  }
}
