use discord::model::Event;

pub trait ReceivesEvents {
  fn receive(&self, event: &Event);
}

pub mod debugger;
pub mod commands;

pub use debugger::*;
pub use commands::*;
