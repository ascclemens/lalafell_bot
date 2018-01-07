use bot::BotEnv;
use database::models::{Message as DbMessage, Edit};

use lalafell::commands::prelude::*;
use lalafell::error::*;

use serenity::builder::CreateEmbed;
use serenity::model::id::ChannelId;

use diesel::prelude::*;

use std::sync::Arc;

const USAGE: &str = "!viewedits <message ID>";

pub struct ViewEditsCommand;

impl ViewEditsCommand {
  pub fn new(_: Arc<BotEnv>) -> Self {
    ViewEditsCommand
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
  fn run(&self, _: &Context, message: &Message, guild: GuildId, _: Arc<RwLock<GuildChannel>>, params: &[&str]) -> CommandResult<'a> {
    let params = self.params(USAGE, params)?;

    let member = match guild.member(message.author.id) {
      Ok(m) => m,
      Err(_) => return Err("You must be a member of the guild to use that command.".into())
    };
    if !member.permissions().chain_err(|| "could not get permissions")?.manage_messages() {
      return Err(ExternalCommandFailure::default()
        .message(|e: CreateEmbed| e
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

    let guild = some_or!(guild.find(), bail!("could not find guild"));
    let channel = match guild.read().channels.get(&ChannelId(*message.channel_id)) {
      Some(c) => Arc::clone(c),
      None => return Err("You cannot view messages not on the current server.".into())
    };

    let discord_message = ChannelId(*message.channel_id).message(params.message_id).ok();

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
    let channel_name = channel.read().name.clone();

    Ok(CommandSuccess::default()
      .message(move |e: CreateEmbed| e
        .title(&format!("Message{} in #{}", author, channel_name))
        .description(&format!("```diff\n{}```", result))))
  }
}
