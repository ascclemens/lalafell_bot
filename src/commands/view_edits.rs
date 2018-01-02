use bot::LalafellBot;
use database::models::{Message as DbMessage, Edit};

use lalafell::bot::Bot;
use lalafell::commands::prelude::*;
use lalafell::error::*;

use discord::model::{Message, LiveServer, PublicChannel, permissions, ChannelId, MessageId};

use diesel::prelude::*;

use std::sync::Arc;

const USAGE: &'static str = "!viewedits <message ID>";

pub struct ViewEditsCommand {
  bot: Arc<LalafellBot>
}

impl ViewEditsCommand {
  pub fn new(bot: Arc<LalafellBot>) -> ViewEditsCommand {
    ViewEditsCommand {
      bot
    }
  }
}

impl HasBot for ViewEditsCommand {
  fn bot(&self) -> &Bot {
    self.bot.as_ref()
  }
}

#[derive(Debug, Deserialize)]
pub struct Params {
  message_id: u64
}

impl HasParams for ViewEditsCommand {
  type Params = Params;
}

impl<'a> PublicChannelCommand<'a> for ViewEditsCommand {
  fn run(&self, message: &Message, server: &LiveServer, channel: &PublicChannel, params: &[&str]) -> CommandResult<'a> {
    let params = self.params(USAGE, params)?;

    let has_perms = server.permissions_for(channel.id, message.author.id).contains(permissions::MANAGE_MESSAGES);
    if !has_perms {
      return Err(ExternalCommandFailure::default()
        .message(|e: EmbedBuilder| e
          .title("Not enough permissions.")
          .description("You don't have enough permissions to use this command."))
        .wrap());
    }

    let message: Option<DbMessage> = ::bot::CONNECTION.with(|c| {
      use database::schema::messages::dsl;
      dsl::messages
        .filter(dsl::message_id.eq(params.message_id.to_string()))
        .first::<DbMessage>(c)
        .optional()
        .chain_err(|| "could not load messages")
    })?;
    let message = match message {
      Some(m) => m,
      None => return Err("No message with that ID recorded.".into())
    };

    let channel = match server.channels.iter().find(|c| c.id.0 == *message.channel_id) {
      Some(c) => c,
      None => return Err("You cannot view messages not on the current server.".into())
    };

    let discord_message = self.bot.discord.get_message(ChannelId(*message.channel_id), MessageId(params.message_id)).ok();

    let edits: Vec<Edit> = ::bot::CONNECTION.with(|c| Edit::belonging_to(&message).load(c).chain_err(|| "could not load edits"))?;
    if edits.is_empty() {
      return Err("That message has not been edited.".into());
    }

    let mut result = String::new();
    let mut messages: Vec<String> = edits.into_iter().map(|x| x.content).collect();
    messages.insert(0, message.content);

    for edits in messages.windows(2) {
      result.push_str("- ");
      result.push_str(&edits[0]);
      result.push('\n');
      result.push_str("+ ");
      result.push_str(&edits[1]);
      result.push_str("\n\n");
    }

    let author = discord_message.map(|m| format!(" by {}", m.author.name)).unwrap_or_default();
    let channel_name = channel.name.clone();

    Ok(CommandSuccess::default()
      .message(move |e: EmbedBuilder| e
        .title(&format!("Message{} in #{}", author, channel_name))
        .description(&format!("```diff\n{}```", result))))
  }
}
