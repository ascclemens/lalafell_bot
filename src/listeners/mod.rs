use LalafellBot;
use config::Listener;

use xivdb::error::*;

use discord::model::Event;

use std::sync::Arc;

pub trait ReceivesEvents {
  fn receive(&self, event: &Event);
}

pub mod debugger;
pub mod commands;
pub mod tag_instructions;
pub mod reaction_authorize;

pub use self::debugger::EventDebugger;
pub use self::commands::CommandListener;
pub use self::tag_instructions::TagInstructions;
pub use self::reaction_authorize::ReactionAuthorize;

pub struct ListenerManager;

impl ListenerManager {
  pub fn from_config(bot: Arc<LalafellBot>, listener: &Listener) -> Result<Box<ReceivesEvents + Send + Sync>> {
    let listener: Box<ReceivesEvents + Send + Sync> = match listener.name.to_lowercase().as_ref() {
      "tag_instructions" => box TagInstructions::new(bot.clone(), listener)?,
      "event_debugger" => box EventDebugger,
      "reaction_authorize" => box ReactionAuthorize::new(bot.clone(), listener)?,
      _ => return Err(format!("no listener called {}", listener.name).into())
    };
    Ok(listener)
  }
}
