use LalafellBot;
use commands::*;

use discord::builders::EmbedBuilder;

use std::sync::Arc;

pub struct ReferenceCountCommand {
  bot: Arc<LalafellBot>
}

impl ReferenceCountCommand {
  pub fn new(bot: Arc<LalafellBot>) -> ReferenceCountCommand {
    ReferenceCountCommand {
      bot: bot
    }
  }
}

impl<'a> Command<'a> for ReferenceCountCommand {
  fn run(&self, message: &Message, _: &[&str]) -> CommandResult<'a> {
    if !self.bot.config.bot.administrators.contains(&message.author.id.0) {
      return Err(ExternalCommandFailure::default()
        .message(|e: EmbedBuilder| e
          .title("Not enough permissions.")
          .description("You don't have enough permissions to use this command."))
        .wrap());
    }
    let strong_references = Arc::strong_count(&self.bot);
    let weak_references = Arc::weak_count(&self.bot);
    Ok(CommandSuccess::default()
      .message(move |e: EmbedBuilder| e.description(&format!("There are currently {} strong references and {} weak references.",
        strong_references,
        weak_references))))
  }
}
