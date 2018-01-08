use bot::LalafellBot;
use tasks::{TaskManager, AutoTagTask, TimeoutCheckTask, DeleteAllMessagesTask, RandomPresenceTask};

use error::*;

use std::sync::Arc;

pub fn tasks(bot: &LalafellBot) -> Result<()> {
  let task_manager = TaskManager::new(Arc::clone(&bot.env));
  task_manager.start_task(AutoTagTask::new());
  task_manager.start_task(TimeoutCheckTask::new());
  task_manager.start_task(DeleteAllMessagesTask::new());
  task_manager.start_task(RandomPresenceTask::new(Arc::clone(&bot.discord.shard_manager)));
  Ok(())
}
