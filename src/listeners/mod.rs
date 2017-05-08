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

pub use debugger::*;
pub use commands::*;
pub use tag_instructions::*;

pub struct ListenerManager;

impl ListenerManager {
  pub fn from_config(bot: Arc<LalafellBot>, listener: &Listener) -> Result<Box<ReceivesEvents + Send>> {
    let listener: Box<ReceivesEvents + Send> = match listener.name.to_lowercase().as_ref() {
      "tag_instructions" => box TagInstructions::new(bot, listener)?,
      "event_debugger" => box EventDebugger,
      _ => return Err(format!("no listener called {}", listener.name).into())
    };
    Ok(listener)
  }
}
