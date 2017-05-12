use LalafellBot;
use commands::*;

use discord::model::PublicChannel;

use std::sync::Arc;

pub struct LeaveCommand {
  bot: Arc<LalafellBot>
}

impl LeaveCommand {
  pub fn new(bot: Arc<LalafellBot>) -> LeaveCommand {
    LeaveCommand {
      bot: bot
    }
  }
}

impl HasBot for LeaveCommand {
  fn bot(&self) -> Arc<LalafellBot> {
    self.bot.clone()
  }
}

impl<'a> PublicChannelCommand<'a> for LeaveCommand {
  fn run(&self, _: &Message, channel: &PublicChannel, _: &[&str]) -> CommandResult<'a> {
    let server_id = channel.server_id;
    {
      let mut connection = self.bot.connection.lock().unwrap();
      let mut voice = connection.voice(Some(server_id));
      voice.disconnect();
    }
    Ok(CommandSuccess::default())
  }
}
