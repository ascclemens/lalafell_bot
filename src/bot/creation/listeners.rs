use LalafellBot;
use listeners::{ListenerManager, CommandListener};
use commands::*;

use xivdb::error::*;

use std::sync::Arc;

pub fn listeners(bot: Arc<LalafellBot>) -> Result<()> {
  let mut listeners = bot.listeners.lock().unwrap();
  listeners.push(box command_listener(bot.clone()));
  for listener in &bot.config.listeners {
    let listener = ListenerManager::from_config(bot.clone(), listener).chain_err(|| format!("could not create listener {}", listener.name))?;
    listeners.push(listener);
  }
  Ok(())
}

fn command_listener<'a>(bot: Arc<LalafellBot>) -> CommandListener<'a> {
  let mut command_listener = CommandListener::new(bot.clone());
  command_listener.add_command(&["race"], box RaceCommand::new(bot.clone()));
  command_listener.add_command(&["tag"], box TagCommand::new(bot.clone()));
  command_listener.add_command(&["autotag"], box AutoTagCommand::new(bot.clone()));
  command_listener.add_command(&["viewtag"], box ViewTagCommand::new(bot.clone()));
  command_listener.add_command(&["updatetags"], box UpdateTagsCommand::new(bot.clone()));
  command_listener.add_command(&["savedatabase"], box SaveDatabaseCommand::new(bot.clone()));
  command_listener.add_command(&["verify"], box VerifyCommand::new(bot.clone()));
  command_listener
}
