use bot::BotEnv;
use config::Task;
use error::Result;

use std::sync::Arc;
use std::thread;

pub trait FromConfig {
  fn from_config(task: &Task) -> Result<Self>
    where Self: Sized;
}

pub trait RunsTask {
  fn start(self, env: Arc<BotEnv>);
}

pub mod delete_all_messages;
pub mod autotag;
pub mod timeout_check;
pub mod random_presence;

pub use self::delete_all_messages::DeleteAllMessagesTask;
pub use self::autotag::AutoTagTask;
pub use self::timeout_check::TimeoutCheckTask;
pub use self::random_presence::RandomPresenceTask;

pub struct TaskManager {
  env: Arc<BotEnv>
}

impl TaskManager {
  pub fn new(env: Arc<BotEnv>) -> Self {
    TaskManager { env }
  }

  pub fn start_from_config(&self, task: &Task) -> Result<()> {
    bail!("no task named {}", task.name);
    // match task.name.to_lowercase().as_str() {
    //   _ => bail!("no task named {}", task.name)
    // }
    // Ok(())
  }

  pub fn start_task<T: RunsTask + Send + 'static>(&self, task: T) {
    let thread_env = Arc::clone(&self.env);
    thread::spawn(move || {
      task.start(thread_env);
    });
  }
}
