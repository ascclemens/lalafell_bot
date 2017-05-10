use LalafellBot;
use commands::*;

use discord::builders::EmbedBuilder;
use discord::model::Channel;

use xivdb::error::*;

use std::sync::Arc;

pub struct ReferenceCountCommand {
  bot: Arc<LalafellBot>
}

impl ReferenceCountCommand {
  pub fn new(bot: Arc<LalafellBot>) -> ReferenceCountCommand {
    ReferenceCountCommand {
      bot: bot
    }
  }
}

impl<'a> Command<'a> for ReferenceCountCommand {
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
    let strong_references = Arc::strong_count(&self.bot);
    let weak_references = Arc::weak_count(&self.bot);
    Ok(CommandSuccess::default()
      .message(move |e: EmbedBuilder| e.description(&format!("There are currently {} strong references and {} weak references.",
        strong_references,
        weak_references))))
  }
}
