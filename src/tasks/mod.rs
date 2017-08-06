use bot::LalafellBot;
use config::Task;
use error::Result;

use std::sync::Arc;
use std::thread;

pub trait FromConfig {
  fn from_config(task: &Task) -> Result<Self>
    where Self: Sized;
}

pub trait RunsTask {
  fn start(self, s: Arc<LalafellBot>);
}

pub mod delete_all_messages;
pub mod autotag;
pub mod timeout_check;

pub use self::delete_all_messages::DeleteAllMessagesTask;
pub use self::autotag::AutoTagTask;
pub use self::timeout_check::TimeoutCheckTask;

pub struct TaskManager {
  bot: Arc<LalafellBot>
}

impl TaskManager {
  pub fn new(bot: Arc<LalafellBot>) -> TaskManager {
    TaskManager {
      bot: bot
    }
  }

  pub fn start_from_config(&self, task: &Task) -> Result<()> {
    bail!("no task named {}", task.name);
    // match task.name.to_lowercase().as_str() {
    //   _ => bail!("no task named {}", task.name)
    // }
    // Ok(())
  }

  pub fn start_task<T: RunsTask + Send + 'static>(&self, task: T) {
    let s = self.bot.clone();
    thread::spawn(move || {
      task.start(s);
    });
  }
}
