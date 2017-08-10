use bot::LalafellBot;

use lalafell::commands::prelude::*;
use lalafell::commands::ChannelOrId;

use discord::model::{Message, LiveServer, PublicChannel, UserId};

use lalafell::bot::Bot;

use std::sync::Arc;

use super::channel;
use super::server;

const USAGE: &'static str = "!configure <subcommand/help>";

pub struct ConfigureCommand {
  bot: Arc<LalafellBot>
}

impl ConfigureCommand {
  pub fn new(bot: Arc<LalafellBot>) -> ConfigureCommand {
    ConfigureCommand {
      bot
    }
  }

  fn help<'a>(&self) -> CommandResult<'a> {
    Ok("Help".into())
  }

  fn channel<'a>(&self, author: UserId, server: &LiveServer, args: &[String]) -> CommandResult<'a> {
    if args.len() < 2 {
      return Err("`!configure channel [channel] [subcommand]`".into());
    }
    let channel = ChannelOrId::parse(&args[0]).map_err(|_| into!(CommandFailure, "Invalid channel reference."))?;
    match args[1].to_lowercase().as_str() {
      "imagedump" | "image_dump" => channel::image_dump(author, channel, server, args),
      _ => Err("Invalid subcommand.".into())
    }
  }

  fn server<'a>(&self, author: UserId, server: &LiveServer, args: &[String], message: &Message) -> CommandResult<'a> {
    if args.is_empty() {
      return Err("`!configure server [subcommand]`".into());
    }
    let subcommand = &args[0];
    let args = &args[1..];
    match subcommand.to_lowercase().as_str() {
      "reaction" | "reactions" => server::reaction(author, server, args),
      "timeoutrole" | "timeout_role" => server::timeout_role(author, server, args),
      "deleteallmessages" | "delete_all_messages" => server::delete_all_messages(author, server, args),
      "autoreply" | "auto_reply" => server::auto_reply(author, server, &message.content),
      _ => Err("Invalid subcommand.".into())
    }
  }
}

impl HasBot for ConfigureCommand {
  fn bot(&self) -> &Bot {
    self.bot.as_ref()
  }
}

#[derive(Debug, Deserialize)]
pub struct Params {
  subcommand: String,
  args: Option<Vec<String>>
}

impl HasParams for ConfigureCommand {
  type Params = Params;
}

impl<'a> PublicChannelCommand<'a> for ConfigureCommand {
  fn run(&self, message: &Message, server: &LiveServer, _: &PublicChannel, params: &[&str]) -> CommandResult<'a> {
    let params = self.params(USAGE, params)?;
    let args = params.args.unwrap_or_default();
    match params.subcommand.to_lowercase().as_str() {
      "help" => self.help(),
      "channel" => self.channel(message.author.id, server, &args),
      "server" => self.server(message.author.id, server, &args, message),
      _ => Err("No such subcommand. Try `help`.".into())
    }
  }
}
