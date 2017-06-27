use bot::LalafellBot;
use listeners::{ListenerManager, Timeouts};
use commands::*;
use error::*;

use lalafell::listeners::CommandListener;

use std::sync::Arc;

pub fn listeners(bot: Arc<LalafellBot>) -> Result<()> {
  let mut listeners = bot.listeners.write().unwrap();
  listeners.push(box command_listener(bot.clone()));
  listeners.push(box Timeouts::new(bot.clone()));
  for listener in &bot.config.listeners {
    let listener = ListenerManager::from_config(bot.clone(), listener).chain_err(|| format!("could not create listener {}", listener.name))?;
    listeners.push(listener);
  }
  Ok(())
}

macro_rules! command_listener {
  (bot => $bot:expr, $($($alias:expr),+ => $name:ident),+) => {{
    let mut command_listener = CommandListener::new($bot.clone());
    $(
      command_listener.add_command(&[$($alias),*], box $name::new($bot.clone()));
    )*
    command_listener
  }}
}

fn command_listener<'a>(bot: Arc<LalafellBot>) -> CommandListener<'a> {
  command_listener! {
    bot => bot,
    "race" => RaceCommand,
    "tag" => TagCommand,
    "autotag" => AutoTagCommand,
    "viewtag" => ViewTagCommand,
    "updatetags" => UpdateTagsCommand,
    "updatetag" => UpdateTagCommand,
    "savedatabase" => SaveDatabaseCommand,
    "verify" => VerifyCommand,
    "referencecount" => ReferenceCountCommand,
    "poll" => PollCommand,
    "pollresults" => PollResultsCommand,
    "timeout" => TimeoutCommand,
    "untimeout" => UntimeoutCommand
  }
}
