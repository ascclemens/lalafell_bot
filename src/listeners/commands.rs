use LalafellBot;
use listeners::ReceivesEvents;
use commands::*;
use discord::model::{Event, Message, ReactionEmoji};

use std::sync::Arc;
use std::collections::HashMap;

pub struct CommandListener<'a> {
  bot: Arc<LalafellBot>,
  commands: HashMap<Vec<String>, Box<Command<'a> + Send + Sync>>
}

impl<'a> CommandListener<'a> {
  pub fn new(bot: Arc<LalafellBot>) -> CommandListener<'a> {
    CommandListener {
      bot: bot,
      commands: Default::default()
    }
  }

  pub fn add_command<T: AsRef<str>>(&mut self, names: &[T], command: Box<Command<'a> + Send + Sync>) {
    self.commands.insert(names.iter().map(|t| t.as_ref().to_string()).collect(), command);
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
    let run_result = command.run(message, params);
    let command_success = run_result.is_ok();
    match run_result {
      Ok(info) => {
        match info.message {
          Some(embed) => {
            let color = if command_success { 0x196358 } else { 0x63191b };
            self.bot.discord.send_embed(message.channel_id, "", |e| embed(e).color(color)).ok();
          },
          None => {
            let emoji = if command_success { "\u{2705}" } else { "\u{274c}" };
            self.bot.discord.add_reaction(message.channel_id, message.id, ReactionEmoji::Unicode(emoji.to_string())).ok();
          }
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
