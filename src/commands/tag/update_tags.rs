use LalafellBot;
use commands::*;
use tasks::AutoTagTask;

use discord::builders::EmbedBuilder;
use discord::model::Channel;

use xivdb::error::*;

use std::sync::Arc;

pub struct UpdateTagsCommand {
  bot: Arc<LalafellBot>
}

impl UpdateTagsCommand {
  pub fn new(bot: Arc<LalafellBot>) -> UpdateTagsCommand {
    UpdateTagsCommand {
      bot: bot
    }
  }
}

impl<'a> Command<'a> for UpdateTagsCommand {
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
    let mut task = AutoTagTask::new();
    task.next_sleep = 0;
    {
      let mut database = self.bot.database.lock().unwrap();
      database.autotags.last_updated = 0;
    }
    ::std::thread::spawn(move || task.run_once(thread_bot));
    Ok(CommandSuccess::default()
      .message(|e: EmbedBuilder| e.description("Task started.")))
  }
}
