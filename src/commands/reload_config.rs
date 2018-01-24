use bot::{self, BotEnv, is_administrator};

use lalafell::commands::prelude::*;

use serenity::builder::CreateEmbed;

#[derive(BotCommand)]
pub struct ReloadConfigCommand {
  env: Arc<BotEnv>
}

impl<'a> Command<'a> for ReloadConfigCommand {
  fn run(&self, _: &Context, message: &Message, _: &[&str]) -> CommandResult<'a> {
    if !is_administrator(&message.author)? {
      return Err(ExternalCommandFailure::default()
        .message(|e: CreateEmbed| e
          .title("Not enough permissions.")
          .description("You don't have enough permissions to use this command."))
        .wrap());
    }
    let config = match bot::config(&self.env.environment) {
      Ok(c) => c,
      Err(e) => return Err(format!("Error reloading config: {}", e).into())
    };
    *self.env.config.write() = config;
    Ok("Config reloaded and updated.".into())
  }
}
