use LalafellBot;
use listeners::ReceivesEvents;
use discord::model::Event;

use std::sync::Arc;

pub struct CommandListener {
  bot: Arc<LalafellBot>
}

impl CommandListener {
  pub fn new(bot: Arc<LalafellBot>) -> CommandListener {
    CommandListener {
      bot: bot
    }
  }
}

impl ReceivesEvents for CommandListener {
  fn receive(&self, event: &Event) {
    let message = match *event {
      Event::MessageCreate(ref m) => m,
      _ => return
    };
    self.bot.check_command(message);
  }
}
