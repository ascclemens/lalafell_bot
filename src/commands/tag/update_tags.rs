use crate::bot::{BotEnv, is_administrator};
use crate::tasks::AutoTagTask;

use lalafell::commands::prelude::*;

use serenity::builder::CreateEmbed;

use std::sync::Arc;

#[derive(BotCommand)]
pub struct UpdateTagsCommand {
  env: Arc<BotEnv>
}

impl<'a> Command<'a> for UpdateTagsCommand {
  fn run(&self, _: &Context, message: &Message, _: &[&str]) -> CommandResult<'a> {
    if !is_administrator(&message.author)? {
      return Err(ExternalCommandFailure::default()
        .message(|e: &mut CreateEmbed| e
          .title("Not enough permissions.")
          .description("You don't have enough permissions to use this command."))
        .wrap());
    }
    let thread_env = Arc::clone(&self.env);
    let mut task = AutoTagTask::new();
    task.next_sleep = 0;
    std::thread::spawn(move || task.run_once(&thread_env));
    Ok("Task started.".into())
  }
}
