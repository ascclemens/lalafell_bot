use LalafellBot;
use commands::*;

use discord::builders::EmbedBuilder;
use discord::model::Channel;

use xivdb::error::*;

use std::sync::Arc;

pub struct SaveDatabaseCommand {
  bot: Arc<LalafellBot>
}

impl SaveDatabaseCommand {
  pub fn new(bot: Arc<LalafellBot>) -> SaveDatabaseCommand {
    SaveDatabaseCommand {
      bot: bot
    }
  }
}

impl<'a> Command<'a> for SaveDatabaseCommand {
  fn run(&self, message: &Message, _: &[&str]) -> CommandResult<'a> {
    let channel = self.bot.discord.get_channel(message.channel_id).chain_err(|| "could not get channel for message")?;
    let server_id = match channel {
      Channel::Public(c) => c.server_id,
      _ => {
        let err: error::Error = "channel was not public".into();
        return Err(err.into());
      }
    };
    let mut state_option = self.bot.state.lock().unwrap();
    let state = state_option.as_mut().unwrap();
    let server = match state.servers().iter().find(|x| x.id == server_id) {
      Some(s) => s,
      None => {
        let err: error::Error = "could not find server for channel".into();
        return Err(err.into());
      }
    };
    if server.owner_id != message.author.id {
      return Err(ExternalCommandFailure::default()
        .message(|e: EmbedBuilder| e
          .title("Not enough permissions.")
          .description("You don't have enough permissions to use this command."))
        .wrap());
    }
    let thread_bot = self.bot.clone();
    ::std::thread::spawn(move || {
      if let Err(e) = thread_bot.save_database(None) {
        error!("Error saving database: {}", e);
      }
    });
    Ok(CommandSuccess::default()
      .message(|e: EmbedBuilder| e.description("Task started.")))
  }
}
