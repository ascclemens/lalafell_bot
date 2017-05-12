use LalafellBot;
use commands::*;

use discord::model::{PublicChannel, ChannelType};
use discord::builders::EmbedBuilder;

use std::sync::Arc;

const USAGE: &'static str = "!join <voice channel name>";

pub struct JoinCommand {
  bot: Arc<LalafellBot>
}

impl JoinCommand {
  pub fn new(bot: Arc<LalafellBot>) -> JoinCommand {
    JoinCommand {
      bot: bot
    }
  }
}

impl HasBot for JoinCommand {
  fn bot(&self) -> Arc<LalafellBot> {
    self.bot.clone()
  }
}

impl<'a> PublicChannelCommand<'a> for JoinCommand {
  fn run(&self, _: &Message, channel: &PublicChannel, params: &[&str]) -> CommandResult<'a> {
    debug!("!join");
    if params.is_empty() {
      debug!("empty params");
      return Err(ExternalCommandFailure::default()
        .message(|e: EmbedBuilder| e
          .title("Not enough parameters.")
          .description(USAGE))
        .wrap());
    }

    let chan_name = params.join(" ");
    debug!("asked to join {}", chan_name);

    let server_id = channel.server_id;
    debug!("server_id: {}", server_id);
    debug!("locking state");
    let state_option = self.bot.state.read().unwrap();
    let state = state_option.as_ref().unwrap();
    let server = match state.servers().iter().find(|x| x.id == server_id) {
      Some(s) => s,
      None => {
        let err: error::Error = "could not find server for channel".into();
        return Err(err.into());
      }
    };
    debug!("found server for text channel");

    let opt_channel = server.channels.iter()
      .find(|x| x.kind == ChannelType::Voice && x.name.to_lowercase() == chan_name.to_lowercase());
    let channel = match opt_channel {
      Some(s) => s,
      None => return Err(ExternalCommandFailure::default()
                .message(move |e: EmbedBuilder| e.
                  description(&format!("Could not find channel {}", chan_name)))
                .wrap())
    };
    debug!("found voice channel");

    {
      debug!("locking connection");
      let mut connection = self.bot.connection.lock().unwrap();
      debug!("connection.voice");
      let mut voice = connection.voice(Some(server_id));
      // FIXME: deadlock if the bot doesn't have voice perms
      debug!("voice.connect");
      voice.connect(channel.id);
    }

    debug!("done");

    Ok(CommandSuccess::default())
  }
}
