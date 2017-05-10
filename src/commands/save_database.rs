use LalafellBot;
use commands::*;

use discord::builders::EmbedBuilder;

use std::sync::Arc;

pub struct SaveDatabaseCommand {
  bot: Arc<LalafellBot>
}

impl SaveDatabaseCommand {
  pub fn new(bot: Arc<LalafellBot>) -> SaveDatabaseCommand {
    SaveDatabaseCommand {
      bot: bot
    }
  }
}

impl<'a> Command<'a> for SaveDatabaseCommand {
  fn run(&self, message: &Message, _: &[&str]) -> CommandResult<'a> {
    if !self.bot.config.bot.administrators.contains(&message.author.id.0) {
      return Err(ExternalCommandFailure::default()
        .message(|e: EmbedBuilder| e
          .title("Not enough permissions.")
          .description("You don't have enough permissions to use this command."))
        .wrap());
    }
    let thread_bot = self.bot.clone();
    ::std::thread::spawn(move || {
      if let Err(e) = thread_bot.save_database(None) {
        error!("Error saving database: {}", e);
      }
    });
    Ok(CommandSuccess::default()
      .message(|e: EmbedBuilder| e.description("Task started.")))
  }
}
