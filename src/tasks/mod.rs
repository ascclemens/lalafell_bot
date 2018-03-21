use bot::BotEnv;

use std::thread;
use std::sync::Arc;

// TODO: Move most tasks to an ephemeral_message style task instead of current setup

pub trait RunsTask {
  fn start(self, env: Arc<BotEnv>);
}

pub mod autotag;
pub mod delete_all_messages;
pub mod ephemeral_messages;
pub mod random_presence;
pub mod role_check;
pub mod tag_queue;
pub mod timeout_check;

pub use self::autotag::AutoTagTask;
pub use self::delete_all_messages::DeleteAllMessagesTask;
pub use self::ephemeral_messages::EphemeralMessageTask;
pub use self::random_presence::RandomPresenceTask;
pub use self::role_check::RoleCheckTask;
pub use self::tag_queue::TagQueueTask;
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
