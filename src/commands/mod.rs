pub mod tag;
pub mod race;
pub mod viewtag;
pub mod save_database;

pub use tag::*;
pub use race::*;
pub use viewtag::*;
pub use save_database::*;

use xivdb::error;

use discord::model::Message;
use discord::builders::EmbedBuilder;

use std::boxed::FnBox;

pub type CommandResult<'a> = Result<CommandSuccess<'a>, CommandFailure<'a>>;

pub trait Command<'a> {
  fn run(&self, message: &Message, params: &[&str]) -> CommandResult<'a>;
}

#[derive(Default)]
pub struct CommandSuccess<'a> {
  pub message: Option<Box<FnBox(EmbedBuilder) -> EmbedBuilder + 'a>>
}

impl<'a> CommandSuccess<'a> {
  pub fn message<F>(mut self, message: F) -> Self
    where F: 'a + FnBox(EmbedBuilder) -> EmbedBuilder
  {
    self.message = Some(box message);
    self
  }
}

pub enum CommandFailure<'a> {
  Internal(InternalCommandFailure),
  External(ExternalCommandFailure<'a>)
}

#[derive(Default)]
pub struct ExternalCommandFailure<'a> {
  pub message: Option<Box<FnBox(EmbedBuilder) -> EmbedBuilder + 'a>>
}

impl<'a> ExternalCommandFailure<'a> {
  pub fn message<F>(mut self, message: F) -> Self
    where F: 'a + FnBox(EmbedBuilder) -> EmbedBuilder + 'static
  {
    self.message = Some(box message);
    self
  }

  pub fn wrap(self) -> CommandFailure<'a> {
    CommandFailure::External(self)
  }
}

#[derive(Debug)]
pub struct InternalCommandFailure {
  pub error: error::Error
}

impl<'a> From<error::Error> for CommandFailure<'a> {
  fn from(error: error::Error) -> Self {
    CommandFailure::Internal(InternalCommandFailure { error: error })
  }
}
