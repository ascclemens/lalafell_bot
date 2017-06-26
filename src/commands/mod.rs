pub mod params;

pub mod tag;
pub mod polling;

pub mod race;
pub mod viewtag;
pub mod save_database;
pub mod verify;
pub mod reference_count;
pub mod timeout;

pub use self::tag::{TagCommand, AutoTagCommand, UpdateTagsCommand, UpdateTagCommand};
pub use self::polling::{PollCommand, PollResultsCommand};
pub use self::timeout::{TimeoutCommand, UntimeoutCommand};

pub use self::race::RaceCommand;
pub use self::viewtag::ViewTagCommand;
pub use self::save_database::SaveDatabaseCommand;
pub use self::verify::VerifyCommand;
pub use self::reference_count::ReferenceCountCommand;

use bot::LalafellBot;

use error::{self, ResultExt};

use discord::model::{Message, LiveServer, Channel, PublicChannel};
use discord::builders::EmbedBuilder;

use serde::de::DeserializeOwned;

use std::boxed::FnBox;
use std::sync::Arc;

pub type CommandResult<'a> = Result<CommandSuccess<'a>, CommandFailure<'a>>;

pub trait Command<'a> {
  fn run(&self, message: &Message, params: &[&str]) -> CommandResult<'a>;
}

pub trait PublicChannelCommand<'a> {
  fn run(&self, message: &Message, server: &LiveServer, channel: &PublicChannel, params: &[&str]) -> CommandResult<'a>;
}

pub trait HasBot {
  fn bot(&self) -> Arc<LalafellBot>;
}

pub trait HasParams {
  type Params: DeserializeOwned;

  fn params<'a>(&self, usage: &str, params: &[&str]) -> Result<Self::Params, CommandFailure<'a>> {
    let string = params.join(" ");
    match params::from_str(&string) {
      Ok(p) => Ok(p),
      Err(::commands::params::error::Error::MissingParams) => {
        let usage = usage.to_owned();
        Err(ExternalCommandFailure::default()
        .message(move |e: EmbedBuilder| e
          .title("Not enough parameters.")
          .description(&usage))
        .wrap())
      },
      Err(e) => Err(e).chain_err(|| "could not parse params")?
    }
  }
}

impl<'a, T> Command<'a> for T
  where T: PublicChannelCommand<'a> + HasBot
{
  fn run(&self, message: &Message, params: &[&str]) -> CommandResult<'a> {
    let channel = self.bot().discord.get_channel(message.channel_id).chain_err(|| "could not get channel for message")?;
    let public_channel = match channel {
      Channel::Public(c) => c,
      _ => return Err("This command must be run in a public channel.".into())
    };
    let server_id = public_channel.server_id;
    let server = {
      let bot = self.bot();
      let state_option = bot.state.read().unwrap();
      let state = state_option.as_ref().unwrap();
      match state.servers().iter().find(|x| x.id == server_id) {
        Some(s) => s.clone(),
        None => {
          let err: error::Error = "could not find server for channel".into();
          return Err(err.into());
        }
      }
    };
    self.run(message, &server, &public_channel, params)
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

impl<'a, T> From<T> for CommandSuccess<'a>
  where T: AsRef<str>
{
  fn from(message: T) -> Self {
    let message = message.as_ref().to_string();
    CommandSuccess::default()
      .message(move |e: EmbedBuilder| e.description(&message))
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

impl<'a, T> From<T> for CommandFailure<'a>
  where T: AsRef<str>
{
  fn from(message: T) -> Self {
    let message = message.as_ref().to_string();
    ExternalCommandFailure::default()
      .message(move |e: EmbedBuilder| e.description(&message))
      .wrap()
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
