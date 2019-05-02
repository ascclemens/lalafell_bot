use crate::tasks::RunsTask;
use crate::bot::BotEnv;
use crate::commands::tag::Tagger;
use lalafell::error::*;
use crate::database::models::Tag;

use serenity::model::id::{UserId, GuildId};

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

  pub fn update_tag(env: &BotEnv, user: UserId, guild: GuildId, character: u64) -> Result<Option<String>> {
    Tagger::tag(env, user, guild, character, false)
  }

  pub fn run_once(&mut self, env: &BotEnv) {
    thread::sleep(Duration::seconds(self.next_sleep).to_std().unwrap());
    self.next_sleep = Duration::minutes(10).num_seconds();
    info!("Autotag task running");
    let mut users: Vec<Tag> = match crate::bot::with_connection(|c| {
      use crate::database::schema::tags::dsl;
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
    for tags in users.chunks_mut(20) {
      for mut tag in tags {
        match AutoTagTask::update_tag(env, UserId(*tag.user_id), GuildId(*tag.server_id), *tag.character_id) {
          Err(e) => {
            warn!("Couldn't update tag for user ID {}: {}", *tag.user_id, e);
            continue;
          },
          Ok(Some(s)) => {
            warn!("Couldn't update tag for user ID {}: {}", *tag.user_id, s);
            continue;
          },
          _ => {},
        }
        tag.last_updated = Utc::now().timestamp();
        let res: ::std::result::Result<Tag, _> = crate::bot::with_connection(|c| tag.save_changes(c));
        if let Err(e) = res {
          warn!("could not update tag last_updated: {}", e);
        }
      }
      thread::sleep(Duration::seconds(1).to_std().unwrap());
    }
    info!("Done updating autotags");
  }
}

impl RunsTask for AutoTagTask {
  fn start(mut self, env: Arc<BotEnv>) {
    info!("Autotag task waiting {} seconds", self.next_sleep);
    loop {
      self.run_once(env.as_ref());
    }
  }
}
