use LalafellBot;
use tasks::RunsTask;
use chrono::prelude::*;
use chrono::Duration;

use std::sync::Arc;
use std::thread;

pub struct DatabaseSaveTask {
  location: String,
  next_sleep: i64
}

impl DatabaseSaveTask {
  pub fn new(location: &str) -> DatabaseSaveTask {
    DatabaseSaveTask {
      location: location.to_string(),
      next_sleep: 0
    }
  }
}

impl RunsTask for DatabaseSaveTask {
  fn start(mut self, s: Arc<LalafellBot>) {
    loop {
      thread::sleep(Duration::seconds(self.next_sleep).to_std().unwrap());
      info!(target: "database_save", "Database save task running");
      let time_to_update = {
        let database = s.database.lock().unwrap();
        database.last_saved + Duration::hours(1).num_seconds() < UTC::now().timestamp()
      };
      if !time_to_update {
        info!(target: "database_save", "Not yet time to save database. Sleeping for five minutes.");
        self.next_sleep = Duration::minutes(5).num_seconds();
        continue;
      }
      if let Err(e) = s.save_database(&self.location) {
        info!(target: "database_save", "could not save database: {}", e);
      }
      {
        let mut database = s.database.lock().unwrap();
        database.last_saved = UTC::now().timestamp();
      }
      info!(target: "database_save", "Database save task done");
    }
  }
}
