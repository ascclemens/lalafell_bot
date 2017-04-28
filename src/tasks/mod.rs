use LalafellBot;

use std::sync::Arc;
use std::thread;

pub trait RunsTask {
  fn start(self, s: Arc<LalafellBot>);
}

pub mod delete_all_messages;
pub mod database_save;
pub mod autotag;

pub use delete_all_messages::*;
pub use database_save::*;
pub use autotag::*;

pub struct TaskManager {
  bot: Arc<LalafellBot>
}

impl TaskManager {
  pub fn new(bot: Arc<LalafellBot>) -> TaskManager {
    TaskManager {
      bot: bot
    }
  }

  pub fn start_task<T: RunsTask + Send + 'static>(&self, task: T) {
    let s = self.bot.clone();
    thread::spawn(move || {
      task.start(s);
    });
  }
}
