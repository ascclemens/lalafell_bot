use tasks::*;
use error::*;
use bot::LalafellBot;

use std::sync::Arc;

pub fn tasks(bot: &LalafellBot) -> Result<()> {
  let task_manager = TaskManager::new(Arc::clone(&bot.env));
  task_manager.start_task(AutoTagTask::new());
  task_manager.start_task(TimeoutCheckTask::new());
  task_manager.start_task(DeleteAllMessagesTask::new());
  task_manager.start_task(RandomPresenceTask::new(Arc::clone(&bot.discord.shard_manager)));
  task_manager.start_task(RoleCheckTask::new());
  task_manager.start_task(TagQueueTask::default());
  task_manager.start_task(EphemeralMessageTask::default());
  task_manager.start_task(TemporaryRolesTask::default());
  Ok(())
}
