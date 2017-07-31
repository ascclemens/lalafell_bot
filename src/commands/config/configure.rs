use bot::LalafellBot;
use database::models::{ChannelConfig, NewChannelConfig, ServerConfig, NewServerConfig, Reaction};

use diesel;
use diesel::prelude::*;

use lalafell::error::*;
use lalafell::commands::prelude::*;
use lalafell::commands::ChannelOrId;

use discord::model::{permissions, Message, LiveServer, PublicChannel, UserId, ChannelId};

use lalafell::bot::Bot;

use std::sync::Arc;

const USAGE: &'static str = "!configure <subcommand/help>";

pub struct ConfigureCommand {
  bot: Arc<LalafellBot>
}

impl ConfigureCommand {
  pub fn new(bot: Arc<LalafellBot>) -> ConfigureCommand {
    ConfigureCommand {
      bot: bot
    }
  }

  fn help<'a>(&self) -> CommandResult<'a> {
    Ok("Help".into())
  }

  fn image_dump<'a>(&self, channel: ChannelId, server: &LiveServer, args: &[String]) -> CommandResult<'a> {
    let config: Option<ChannelConfig> = ::bot::CONNECTION.with(|c| {
      use database::schema::channel_configs::dsl;
      dsl::channel_configs
        .filter(dsl::server_id.eq(server.id.0.to_string()).and(dsl::channel_id.eq(channel.to_string())))
        .first(c)
        .optional()
        .chain_err(|| "could not load channel configs")
    })?;
    if args.len() < 3 {
      let status = match config.and_then(|c| c.image_dump_allowed) {
        Some(true) => "enabled",
        Some(false) => "disabled",
        None => "unset (disabled)"
      };
      return Ok(format!("`!imagedump` status in {}: {}", channel.mention(), status).into());
    }
    let enabled = match args[2].to_lowercase().as_str() {
      "enabled" | "enable" | "on" | "true" | "yes" => true,
      "disabled" | "disable" | "off" | "false" | "no" => false,
      _ => return Err("Unknown enabled state provided".into())
    };
    match config {
      Some(conf) => {
        ::bot::CONNECTION.with(|c| {
          use database::schema::channel_configs::dsl;
          diesel::update(&conf)
            .set(dsl::image_dump_allowed.eq(Some(enabled)))
            .execute(c)
            .chain_err(|| "could not update config")
        })?;
      },
      None => {
        ::bot::CONNECTION.with(|c| {
          let new = NewChannelConfig {
            server_id: server.id.into(),
            channel_id: channel.into(),
            image_dump_allowed: Some(enabled)
          };
          diesel::insert(&new)
            .into(::database::schema::channel_configs::table)
            .execute(c)
            .chain_err(|| "could not add config")
        })?;
      }
    }
    Ok(format!("Set `!imagedump` status in {} to {}.", channel.mention(), if enabled { "enabled" } else { "disabled" }).into())
  }

  fn reactions<'a>(&self, channel: ChannelId, server: &LiveServer, args: &[String]) -> CommandResult<'a> {
    if args.len() < 3 {
      let reactions: Vec<Reaction> = ::bot::CONNECTION.with(|c| {
        use database::schema::reactions::dsl;
        dsl::reactions.load(c).chain_err(|| "could not load reactions")
      })?;
      let strings: Vec<String> = reactions.iter()
        .map(|r| format!("{}. {} grants `{}` on {} in {}", r.id, r.emoji, r.role, *r.message_id, ChannelId(*r.channel_id).mention()))
        .collect();
      return Ok(strings.join("\n").into());
    }
    let subcommand = match args[2].to_lowercase().as_str() {
      "add" | "create" => {},
      "remove" | "delete" => {},
      _ => return Err("Invalid subcommand.".into())
    };
    return Err("Unimplemented.".into());
  }

  fn channel<'a>(&self, author: UserId, server: &LiveServer, args: &[String]) -> CommandResult<'a> {
    if args.len() < 2 {
      return Err("`!configure channel [channel] [subcommand]`".into());
    }
    let channel = ChannelOrId::parse(&args[0]).map_err(|_| into!(CommandFailure, "Invalid channel reference."))?;
    // FIXME: imagedump is manage messages, but reactions is manage roles
    let has_permissions = server.permissions_for(channel, author).contains(permissions::MANAGE_MESSAGES);
    if !has_permissions {
      return Err(ExternalCommandFailure::default()
        .message(|e: EmbedBuilder| e
          .title("Not enough permissions.")
          .description("You don't have enough permissions to use this command."))
        .wrap());
    }
    match args[1].to_lowercase().as_str() {
      "imagedump" | "image_dump" => self.image_dump(channel, server, args),
      "reaction" | "reactions" => self.reactions(channel, server, args),
      _ => return Err("Invalid subcommand.".into())
    }
  }

  fn server<'a>(&self, author: UserId, server: &LiveServer, args: &[String]) -> CommandResult<'a> {
    if args.is_empty() {
      return Err("`!configure server [subcommand]`".into());
    }
    if author != server.owner_id {
      return Err(ExternalCommandFailure::default()
        .message(|e: EmbedBuilder| e
          .title("Not enough permissions.")
          .description("You don't have enough permissions to use this command."))
        .wrap());
    }
    match args[0].to_lowercase().as_str() {
      "timeoutrole" | "timeout_role" => {} // TODO: move into separate methods,
      _ => return Err("Invalid subcommand.".into())
    }
    let config: Option<ServerConfig> = ::bot::CONNECTION.with(|c| {
      use database::schema::server_configs::dsl;
      dsl::server_configs
        .filter(dsl::server_id.eq(server.id.0.to_string()))
        .first(c)
        .optional()
        .chain_err(|| "could not load channel configs")
    })?;
    if args.len() < 2 {
      let status = match config.and_then(|c| c.timeout_role) {
        Some(role) => role,
        None => String::from("unset (disabled)")
      };
      return Ok(format!("Timeout role on {}: {}", server.name, status).into());
    }
    let role_name = args[1].to_lowercase();
    let role = match server.roles.iter().find(|r| r.name.to_lowercase() == role_name) {
      Some(r) => r,
      None => return Err(format!("No role by the name `{}`.", &args[2]).into())
    };
    match config {
      Some(conf) => {
        ::bot::CONNECTION.with(|c| {
          use database::schema::server_configs::dsl;
          diesel::update(&conf)
            .set(dsl::timeout_role.eq(Some(role.name.clone())))
            .execute(c)
            .chain_err(|| "could not update config")
        })?;
      },
      None => {
        ::bot::CONNECTION.with(|c| {
          let new = NewServerConfig {
            server_id: server.id.into(),
            timeout_role: Some(role.name.clone())
          };
          diesel::insert(&new)
            .into(::database::schema::server_configs::table)
            .execute(c)
            .chain_err(|| "could not add config")
        })?;
      }
    }
    Ok(format!("Set timeout role in {} to {}.", server.name, role.name).into())
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
      "server" => self.server(message.author.id, server, &args),
      _ => Err("No such subcommand. Try `help`.".into())
    }
  }
}
