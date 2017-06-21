use bot::LalafellBot;
use commands::*;
use tasks::AutoTagTask;

use discord::builders::EmbedBuilder;

use std::sync::Arc;

pub struct UpdateTagsCommand {
  bot: Arc<LalafellBot>
}

impl UpdateTagsCommand {
  pub fn new(bot: Arc<LalafellBot>) -> UpdateTagsCommand {
    UpdateTagsCommand {
      bot: bot
    }
  }
}

impl<'a> Command<'a> for UpdateTagsCommand {
  fn run(&self, message: &Message, _: &[&str]) -> CommandResult<'a> {
    if !self.bot.config.bot.administrators.contains(&message.author.id.0) {
      return Err(ExternalCommandFailure::default()
        .message(|e: EmbedBuilder| e
          .title("Not enough permissions.")
          .description("You don't have enough permissions to use this command."))
        .wrap());
    }
    let thread_bot = self.bot.clone();
    let mut task = AutoTagTask::new();
    task.next_sleep = 0;
    {
      let mut database = self.bot.database.write().unwrap();
      database.autotags.last_updated = 0;
    }
    ::std::thread::spawn(move || task.run_once(thread_bot));
    Ok("Task started.".into())
  }
}
