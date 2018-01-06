use bot::BotEnv;

use lalafell::commands::prelude::*;

pub struct ReferenceCountCommand {
  env: Arc<BotEnv>
}

impl ReferenceCountCommand {
  pub fn new(env: Arc<BotEnv>) -> ReferenceCountCommand {
    ReferenceCountCommand { env }
  }
}

impl<'a> Command<'a> for ReferenceCountCommand {
  fn run(&self, _: &Context, message: &Message, _: &[&str]) -> CommandResult<'a> {
    if !self.env.config.bot.administrators.contains(&message.author.id.0) {
      return Err(ExternalCommandFailure::default()
        .message(|e: CreateEmbed| e
          .title("Not enough permissions.")
          .description("You don't have enough permissions to use this command."))
        .wrap());
    }
    let strong_references = Arc::strong_count(&self.env);
    let weak_references = Arc::weak_count(&self.env);
    Ok(format!("There are currently {} strong references and {} weak references.",
                                                             strong_references,
                                                             weak_references).into())
  }
}
