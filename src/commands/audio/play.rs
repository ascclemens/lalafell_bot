use LalafellBot;
use commands::*;

use discord::voice;
use discord::model::PublicChannel;
use discord::builders::EmbedBuilder;

use std::sync::Arc;

const USAGE: &'static str = "!play <youtube url/ID>";

pub struct PlayCommand {
  bot: Arc<LalafellBot>
}

impl PlayCommand {
  pub fn new(bot: Arc<LalafellBot>) -> PlayCommand {
    PlayCommand {
      bot: bot
    }
  }
}

impl HasBot for PlayCommand {
  fn bot(&self) -> Arc<LalafellBot> {
    self.bot.clone()
  }
}

impl<'a> PublicChannelCommand<'a> for PlayCommand {
  fn run(&self, _: &Message, channel: &PublicChannel, params: &[&str]) -> CommandResult<'a> {
    if params.is_empty() {
      return Err(ExternalCommandFailure::default()
        .message(|e: EmbedBuilder| e
          .title("Not enough parameters.")
          .description(USAGE))
        .wrap());
    }

    let url = params.join(" ");

    let ytdl_stream = match voice::open_ytdl_stream(&url) {
      Ok(y) => y,
      Err(_) => return Err(ExternalCommandFailure::default()
                  .message(|e: EmbedBuilder| e
                    .description("Could not open that URL for playing."))
                  .wrap())
    };

    {
      let mut connection = self.bot.connection.lock().unwrap();
      let mut voice = connection.voice(Some(channel.server_id));
      voice.play(ytdl_stream);
    }

    Ok(CommandSuccess::default())
  }
}
