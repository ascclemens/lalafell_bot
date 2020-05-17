use lalafell::error::*;
use lalafell::commands::prelude::*;
use lalafell::commands::ChannelOrId;

use serenity::client::Context;
use serenity::model::id::{ChannelId, GuildId};
use serenity::model::guild::{Emoji, Member, Role};
use serenity::model::channel::{Message, GuildChannel, Channel};

use chrono::Utc;

use serde_json;

use std::fs::{self, File};
use std::path::Path;

#[derive(BotCommand)]
pub struct ArchiveCommand;

#[derive(Debug, StructOpt)]
#[structopt(help = "Archive a channel")]
pub struct Params {
  #[structopt(help = "The channel to archive")]
  channel: ChannelOrId
}

impl HasParams for ArchiveCommand {
  type Params = Params;
}

impl<'a> PublicChannelCommand<'a> for ArchiveCommand {
  fn run(&self, ctx: &Context, message: &Message, _: GuildId, _: Arc<RwLock<GuildChannel>>, params: &[&str]) -> CommandResult<'a> {
    let params = self.params_then("archive", params, |a| a.setting(structopt::clap::AppSettings::ArgRequiredElseHelp))?;
    let channel = match params.channel.to_channel(ctx).chain_err(|| "could not get channel")? {
      Channel::Guild(g) => g,
      _ => return Err("This channel must be a guild channel.".into())
    };
    let channel = channel.read();
    let member = match channel.guild_id.member(ctx, message.author.id) {
      Ok(m) => m,
      Err(_) => return Err("You must be a member of the guild to archive its channels.".into())
    };
    if !member.permissions(&ctx).chain_err(|| "could not get permissions")?.manage_channels() {
      return Err(ExternalCommandFailure::default()
        .message(|e: &mut CreateEmbed| e
          .title("Not enough permissions.")
          .description("You don't have enough permissions to use this command."))
        .wrap());
    }

    let guild = channel.guild_id.to_guild_cached(&ctx).chain_err(|| "could not find guild")?;

    let archive_path = Path::new("./archives")
      .join(&guild.read().id.to_string())
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

    let mut messages = channel.messages(&ctx, |gm| gm.limit(100)).chain_err(|| "could not download first set of messages")?;
    if messages.len() >= 100 {
      loop {
        let last_message_id = messages[messages.len() - 1].id;
        let mut next_batch = channel.messages(&ctx, |gm| gm.before(last_message_id)).chain_err(|| "could not download more messages")?;
        if next_batch.is_empty() {
          break;
        }
        messages.append(&mut next_batch);
      }
    }

    let num_messages = messages.len();

    let guild = guild.read();
    let archive = Archive {
      timestamp: Utc::now().timestamp(),
      server: ArchiveServer {
        name: guild.name.clone(),
        roles: guild.roles.values().cloned().collect(),
        members: guild.members.values().cloned().collect(),
        channels: guild.channels.iter()
          .map(|(_, c)| {
            let c = c.read();
            ArchiveChannel { id: c.id, name: c.name.clone(), topic: c.topic.clone() }
          })
          .collect(),
        icon: guild.icon.clone(),
        emojis: guild.emojis.values().cloned().collect()
      },
      channel: ArchiveChannel {
        id: channel.id,
        name: channel.name.clone(),
        topic: channel.topic.clone()
      },
      messages
    };

    let file = File::create(archive_path)
      .chain_err(|| "could not create archive file")?;
    serde_json::to_writer(file, &archive)
      .chain_err(|| "could not serialize messages")?;

    Ok(format!("Archived {} message{}.", num_messages, if num_messages == 1 { "" } else { "s" }).into())
  }
}

#[derive(Debug, Serialize)]
struct Archive {
  timestamp: i64,
  server: ArchiveServer,
  channel: ArchiveChannel,
  messages: Vec<Message>
}

#[derive(Debug, Serialize)]
struct ArchiveServer {
  name: String,
  roles: Vec<Role>,
  members: Vec<Member>,
  channels: Vec<ArchiveChannel>,
  icon: Option<String>,
  emojis: Vec<Emoji>
}

#[derive(Debug, Serialize)]
struct ArchiveChannel {
  id: ChannelId,
  name: String,
  topic: Option<String>
}
