use bot::BotEnv;

use chrono::{Utc, DateTime};

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

pub struct Wait<T> {
  inner: T,
  now: DateTime<Utc>,
  last: i64
}

impl<T, R> Wait<T>
  where T: Iterator<Item=(i64, R)>
{
  pub fn new(inner: T) -> Self {
    Self {
      inner,
      now: Utc::now(),
      last: 0
    }
  }
}

impl<T, R> Iterator for Wait<T>
  where T: Iterator<Item=(i64, R)>
{
  type Item = (i64, R);

  fn next(&mut self) -> Option<Self::Item> {
    let item = self.inner.next()?;
    if item.0 <= self.now.timestamp() {
      return Some((0, item.1));
    }
    if self.last == 0 {
      self.last = self.now.timestamp();
    }
    let wait = item.0 - self.last;
    self.last += wait;
    Some((wait, item.1))
  }
}
