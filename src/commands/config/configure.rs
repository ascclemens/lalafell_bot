use bot::BotEnv;

use lalafell::commands::prelude::*;
use lalafell::commands::ChannelOrId;

use serenity::model::id::UserId;

use super::channel;
use super::server;

const USAGE: &str = "!configure <subcommand/help>";

pub struct ConfigureCommand;

impl ConfigureCommand {
  pub fn new(_: Arc<BotEnv>) -> Self {
    ConfigureCommand
  }

  fn help<'a>(&self) -> CommandResult<'a> {
    Ok("Help".into())
  }

  fn channel<'a>(&self, author: UserId, guild: GuildId, args: &[String]) -> CommandResult<'a> {
    if args.len() < 2 {
      return Err("`!configure channel [channel] [subcommand]`".into());
    }
    let channel = ChannelOrId::parse(&args[0]).map_err(|_| into!(CommandFailure, "Invalid channel reference."))?;
    match args[1].to_lowercase().as_str() {
      "imagedump" | "image_dump" => channel::image_dump(author, channel, guild, args),
      _ => Err("Invalid subcommand.".into())
    }
  }

  fn server<'a>(&self, author: UserId, guild: GuildId, args: &[String], message: &Message) -> CommandResult<'a> {
    if args.is_empty() {
      return Err("`!configure server [subcommand]`".into());
    }
    let subcommand = &args[0];
    let args = &args[1..];
    match subcommand.to_lowercase().as_str() {
      "reaction" | "reactions" => server::reaction(author, guild, args),
      "timeoutrole" | "timeout_role" => server::timeout_role(author, guild, args),
      "deleteallmessages" | "delete_all_messages" => server::delete_all_messages(author, guild, args),
      "autoreply" | "auto_reply" => server::auto_reply(author, guild, &message.content),
      _ => Err("Invalid subcommand.".into())
    }
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
  fn run(&self, _: &Context, message: &Message, guild: GuildId, _: Arc<RwLock<GuildChannel>>, params: &[&str]) -> CommandResult<'a> {
    let params = self.params(USAGE, params)?;
    let args = params.args.unwrap_or_default();
    match params.subcommand.to_lowercase().as_str() {
      "help" => self.help(),
      "channel" => self.channel(message.author.id, guild, &args),
      "server" => self.server(message.author.id, guild, &args, message),
      _ => Err("No such subcommand. Try `help`.".into())
    }
  }
}
