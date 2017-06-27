use bot::LalafellBot;
use tasks::RunsTask;
use commands::tag::Tagger;
use lalafell::error::*;

use discord::State;
use discord::model::{UserId, ServerId};
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

  pub fn update_tag(s: &LalafellBot, state: &State, user: UserId, server: ServerId, character: u64) -> Result<Option<String>> {
    let server = match state.servers().iter().find(|s| s.id.0 == server.0) {
      Some(ser) => ser,
      None => return Ok(Some(format!("Couldn't find server for user ID {}", user.0)))
    };
    Tagger::tag(s, user, server, character, false)
  }

  pub fn run_once(&mut self, s: Arc<LalafellBot>) {
    thread::sleep(Duration::seconds(self.next_sleep).to_std().unwrap());
    info!("Autotag task running");
    let time_to_update = {
      let database = s.database.read().unwrap();
      database.autotags.last_updated + Duration::hours(12).num_seconds() < Utc::now().timestamp()
    };
    if !time_to_update {
      info!("Not yet time to update, sleeping 30 minutes");
      self.next_sleep = Duration::minutes(30).num_seconds();
      return;
    }
    info!("Time to update autotags");
    let users: Vec<(UserId, ServerId, u64)> = {
      let database = s.database.read().unwrap();
      database.autotags.users.iter().map(|u| (UserId(u.user_id), ServerId(u.server_id), u.character_id)).collect()
    };
    {
      let option_state = s.state.read().unwrap();
      let state = match option_state.as_ref() {
        Some(st) => st,
        None => {
          info!("Bot not connected. Will try again in 30 seconds.");
          self.next_sleep = 30;
          return;
        }
      };
      for (user_id, server_id, character_id) in users {
        if let Err(e) = AutoTagTask::update_tag(s.as_ref(), state, user_id, server_id, character_id) {
          warn!("Couldn't update tag for user ID {}: {}", user_id, e);
        }
      }
    }
    {
      let mut database = s.database.write().unwrap();
      database.autotags.last_updated = Utc::now().timestamp();
    };
    info!("Done updating autotags");
  }
}

impl RunsTask for AutoTagTask {
  fn start(mut self, s: Arc<LalafellBot>) {
    info!("Autotag task waiting {} seconds", self.next_sleep);
    loop {
      self.run_once(s.clone());
    }
  }
}
