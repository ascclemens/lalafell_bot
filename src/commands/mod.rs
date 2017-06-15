pub mod tag;
pub mod polling;

pub mod race;
pub mod viewtag;
pub mod save_database;
pub mod verify;
pub mod reference_count;
pub mod timeout;

pub use self::tag::{TagCommand, AutoTagCommand, UpdateTagsCommand};
pub use self::polling::{PollCommand, PollResultsCommand};
pub use self::timeout::{TimeoutCommand, UntimeoutCommand};

pub use self::race::RaceCommand;
pub use self::viewtag::ViewTagCommand;
pub use self::save_database::SaveDatabaseCommand;
pub use self::verify::VerifyCommand;
pub use self::reference_count::ReferenceCountCommand;

use bot::LalafellBot;

use error::{self, ResultExt};

use discord::model::{Message, Channel, PublicChannel};
use discord::builders::EmbedBuilder;

use std::boxed::FnBox;
use std::sync::Arc;

pub type CommandResult<'a> = Result<CommandSuccess<'a>, CommandFailure<'a>>;

pub trait Command<'a> {
  fn run(&self, message: &Message, params: &[&str]) -> CommandResult<'a>;
}

pub trait PublicChannelCommand<'a> {
  fn run(&self, message: &Message, channel: &PublicChannel, params: &[&str]) -> CommandResult<'a>;
}

pub trait HasBot {
  fn bot(&self) -> Arc<LalafellBot>;
}

impl<'a, T> Command<'a> for T
  where T: PublicChannelCommand<'a> + HasBot
{
  fn run(&self, message: &Message, params: &[&str]) -> CommandResult<'a> {
    let channel = self.bot().discord.get_channel(message.channel_id).chain_err(|| "could not get channel for message")?;
    let public_channel = match channel {
      Channel::Public(c) => c,
      _ => return Err(ExternalCommandFailure::default()
        .message(|e: EmbedBuilder| e.description("This command must be run in a public channel."))
        .wrap())
    };
    self.run(message, &public_channel, params)
  }
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
