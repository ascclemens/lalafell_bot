use bot::BotEnv;

use std::thread;
use std::sync::Arc;

pub trait RunsTask {
  fn start(self, env: Arc<BotEnv>);
}

pub mod autotag;
pub mod delete_all_messages;
pub mod random_presence;
pub mod role_check;
pub mod timeout_check;

pub use self::autotag::AutoTagTask;
pub use self::delete_all_messages::DeleteAllMessagesTask;
pub use self::random_presence::RandomPresenceTask;
pub use self::role_check::RoleCheckTask;
pub use self::timeout_check::TimeoutCheckTask;

pub struct TaskManager {
  env: Arc<BotEnv>
}

impl TaskManager {
  pub fn new(env: Arc<BotEnv>) -> Self {
    TaskManager { env }
  }

  pub fn start_task<T: RunsTask + Send + 'static>(&self, task: T) {
    let thread_env = Arc::clone(&self.env);
    thread::spawn(move || {
      task.start(thread_env);
    });
  }
}
