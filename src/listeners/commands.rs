use LalafellBot;
use listeners::ReceivesEvents;
use commands::*;
use discord::model::{Event, Message};

use std::sync::Arc;
use std::collections::HashMap;

pub struct CommandListener<'a> {
  bot: Arc<LalafellBot>,
  commands: HashMap<Vec<String>, Box<Command<'a> + Send>>
}

impl<'a> CommandListener<'a> {
  pub fn new(bot: Arc<LalafellBot>) -> CommandListener<'a> {
    CommandListener {
      bot: bot,
      commands: Default::default()
    }
  }

  pub fn commands(&mut self) -> &mut HashMap<Vec<String>, Box<Command<'a> + Send>> {
    &mut self.commands
  }

  fn check_command(&self, message: &Message) {
    let parts: Vec<&str> = message.content.split_whitespace().collect();
    if parts.is_empty() {
      return;
    }
    let first = parts[0];
    if !first.starts_with('!') {
      return;
    }
    let command_name = &first[1..].to_lowercase();
    let params = &parts[1..];
    let (_, command) = match self.commands.iter().find(|&(names, _)| names.contains(&command_name.to_lowercase())) {
      Some(c) => c,
      None => return
    };
    // FIXME: add reactions
    match command.run(message, params) {
      Ok(info) => {
        if let Some(embed) = info.message {
          self.bot.discord.send_embed(message.channel_id, "", embed).ok();
        }
      },
      Err(CommandFailure::Internal(info)) => {
        self.bot.discord.send_embed(message.channel_id, "",
          |e| e.description("An internal error happened while processing this command.")).ok();
        for err in info.error.iter() {
          error!("error: {:#?}", err);
        }
      },
      Err(CommandFailure::External(info)) => {
        if let Some(embed) = info.message {
          self.bot.discord.send_embed(message.channel_id, "", embed).ok();
        }
      }
    }
  }
}

impl<'a> ReceivesEvents for CommandListener<'a> {
  fn receive(&self, event: &Event) {
    let message = match *event {
      Event::MessageCreate(ref m) => m,
      _ => return
    };
    self.check_command(message);
  }
}
