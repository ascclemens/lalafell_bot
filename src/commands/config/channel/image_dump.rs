use database::models::{ChannelConfig, NewChannelConfig};

use discord::model::{permissions, UserId, ChannelId, LiveServer};

use diesel;
use diesel::prelude::*;

use lalafell::error::*;
use lalafell::commands::prelude::*;

pub fn image_dump<'a>(author: UserId, channel: ChannelId, server: &LiveServer, args: &[String]) -> CommandResult<'a> {
  if !server.permissions_for(channel, author).contains(permissions::MANAGE_MESSAGES) {
    return Err(ExternalCommandFailure::default()
      .message(|e: EmbedBuilder| e
        .title("Not enough permissions.")
        .description("You don't have enough permissions to use this command."))
      .wrap());
  }
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
    Some(mut conf) => {
      conf.image_dump_allowed = Some(enabled);
      ::bot::CONNECTION.with(|c| conf.save_changes::<ChannelConfig>(c).chain_err(|| "could not update config"))?;
    },
    None => {
      ::bot::CONNECTION.with(|c| {
        let new = NewChannelConfig {
          server_id: server.id.into(),
          channel_id: channel.into(),
          image_dump_allowed: Some(enabled)
        };
        diesel::insert_into(::database::schema::channel_configs::table)
          .values(&new)
          .execute(c)
          .chain_err(|| "could not add config")
      })?;
    }
  }
  Ok(format!("Set `!imagedump` status in {} to {}.", channel.mention(), if enabled { "enabled" } else { "disabled" }).into())
}
