use bot::LalafellBot;
use tasks::{TaskManager, AutoTagTask, TimeoutCheckTask, DeleteAllMessagesTask};

use error::*;

pub fn tasks(bot: &LalafellBot) -> Result<()> {
  let task_manager = TaskManager::new(bot.env.clone());
  task_manager.start_task(AutoTagTask::new());
  task_manager.start_task(TimeoutCheckTask::new());
  task_manager.start_task(DeleteAllMessagesTask::new());
  for task in &bot.env.config.tasks {
    task_manager.start_from_config(task).chain_err(|| format!("could not create task {}", task.name))?;
  }
  Ok(())
}
