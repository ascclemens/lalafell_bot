use bot::LalafellBot;

use lalafell::bot::Bot;
use lalafell::commands::prelude::*;
use lalafell::commands::ChannelOrId;
use lalafell::error::*;

use discord::GetMessages;
use discord::model::{permissions, Message, PublicChannel};


use serde_json;

use std::sync::Arc;
use std::fs::{self, File};
use std::path::Path;

const USAGE: &'static str = "!archive <channel>";

pub struct ArchiveCommand {
  bot: Arc<LalafellBot>
}

impl ArchiveCommand {
  pub fn new(bot: Arc<LalafellBot>) -> ArchiveCommand {
    ArchiveCommand {
      bot: bot
    }
  }
}

impl HasBot for ArchiveCommand {
  fn bot(&self) -> &Bot {
    self.bot.as_ref()
  }
}

#[derive(Debug, Deserialize)]
pub struct Params {
  channel: ChannelOrId
}

impl HasParams for ArchiveCommand {
  type Params = Params;
}

impl<'a> PublicChannelCommand<'a> for ArchiveCommand {
  fn run(&self, message: &Message, server: &LiveServer, _: &PublicChannel, params: &[&str]) -> CommandResult<'a> {
    let params = self.params(USAGE, params)?;
    if !server.channels.iter().any(|c| c.id == *params.channel) {
      return Err("This command must be run in the server the channel is in.".into());
    }
    let can_manage_chans = server.permissions_for(*params.channel, message.author.id).contains(permissions::MANAGE_CHANNELS);
    if !can_manage_chans {
      return Err(ExternalCommandFailure::default()
        .message(|e: EmbedBuilder| e
          .title("Not enough permissions.")
          .description("You don't have enough permissions to use this command."))
        .wrap());
    }

    let archive_path = Path::new("./archives")
      .join(&server.id.to_string())
      .join(&params.channel.to_string())
      .with_extension("json");
    if let Some(parent) = archive_path.parent() {
      if !parent.exists() {
        fs::create_dir_all(&parent)
          .chain_err(|| "could not create archive directory")?;
      }
    }
    if archive_path.exists() {
      return Err("This channel is already archived.".into());
    }

    let mut messages = self.bot.discord.get_messages(*params.channel, GetMessages::MostRecent, Some(100))
      .chain_err(|| "could not download first set of messages")?;
    if messages.len() >= 100 {
      loop {
        let last_message_id = messages[messages.len() - 1].id;
        let mut next_batch = self.bot.discord.get_messages(*params.channel, GetMessages::Before(last_message_id), Some(100))
          .chain_err(|| "could not download more messages")?;
        if next_batch.is_empty() {
          break;
        }
        messages.append(&mut next_batch);
      }
    }

    let file = File::create(archive_path)
      .chain_err(|| "could not create archive file")?;
    serde_json::to_writer(file, &messages)
      .chain_err(|| "could not serialize messages")?;

    Ok(format!("Archived {} message{}.", messages.len(), if messages.len() == 1 { "" } else { "s" }).into())
  }
}
