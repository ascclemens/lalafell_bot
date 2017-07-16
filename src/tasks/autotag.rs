use bot::LalafellBot;
use tasks::RunsTask;
use commands::tag::Tagger;
use lalafell::error::*;
use database::models::Tag;

use discord::State;
use discord::model::{UserId, ServerId};

use chrono::Duration;
use chrono::Utc;

use diesel::prelude::*;

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
    self.next_sleep = Duration::minutes(10).num_seconds();
    info!("Autotag task running");
    let users: Vec<Tag> = match ::bot::CONNECTION.with(|c| {
      use database::schema::tags::dsl;
      let twelve_hours_ago = Utc::now().timestamp() - Duration::hours(12).num_seconds();
      dsl::tags
        .filter(dsl::last_updated.lt(twelve_hours_ago))
        .load(c)
    }) {
      Ok(t) => t,
      Err(e) => {
        warn!("could not load tags: {}", e);
        return;
      }
    };
    info!("{} tag{} to update", users.len(), if users.len() == 1 { "" } else { "s" });
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
      for tag in users {
        if let Err(e) = AutoTagTask::update_tag(s.as_ref(), state, UserId(*tag.user_id), ServerId(*tag.server_id), *tag.character_id) {
          warn!("Couldn't update tag for user ID {}: {}", *tag.user_id, e);
          continue;
        }
        ::bot::CONNECTION.with(|c| {
          use database::schema::tags::dsl;
          let res = ::diesel::update(&tag)
            .set(dsl::last_updated.eq(Utc::now().timestamp()))
            .execute(c);
          if let Err(e) = res {
            warn!("could not update tag last_updated: {}", e);
          }
        });
      }
    }
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
