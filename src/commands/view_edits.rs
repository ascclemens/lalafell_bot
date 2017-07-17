use bot::LalafellBot;
use database::models::{Message as DbMessage, Edit};

use lalafell::bot::Bot;
use lalafell::commands::prelude::*;
use lalafell::error::*;

use discord::model::{Message, LiveServer, PublicChannel, permissions};

use diesel::prelude::*;

use std::sync::Arc;

const USAGE: &'static str = "!viewedits <message ID>";

pub struct ViewEditsCommand {
  bot: Arc<LalafellBot>
}

impl ViewEditsCommand {
  pub fn new(bot: Arc<LalafellBot>) -> ViewEditsCommand {
    ViewEditsCommand {
      bot: bot
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
      None => return Err("No message with that ID recoreded.".into())
    };

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

    Ok(format!("```diff\n{}```", result).into())
  }
}
