use bot::LalafellBot;
use tasks::{TaskManager, AutoTagTask, TimeoutCheckTask, DeleteAllMessagesTask};

use error::*;

use std::sync::Arc;

pub fn tasks(bot: Arc<LalafellBot>) -> Result<()> {
  let task_manager = TaskManager::new(bot.clone());
  task_manager.start_task(AutoTagTask::new());
  task_manager.start_task(TimeoutCheckTask::new());
  task_manager.start_task(DeleteAllMessagesTask::new());
  for task in &bot.config.tasks {
    task_manager.start_from_config(task).chain_err(|| format!("could not create task {}", task.name))?;
  }
  Ok(())
}
