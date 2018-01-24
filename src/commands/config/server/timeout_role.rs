use database::models::{ToU64, ServerConfig, NewServerConfig};

use serenity::builder::CreateEmbed;
use serenity::model::id::UserId;

use diesel;
use diesel::prelude::*;

use lalafell::error::*;
use lalafell::commands::prelude::*;

pub fn timeout_role<'a>(author: UserId, guild: GuildId, args: &[String]) -> CommandResult<'a> {
  let guild = guild.find().chain_err(|| "could not find guild")?;
  if author != guild.read().owner_id {
    return Err(ExternalCommandFailure::default()
      .message(|e: CreateEmbed| e
        .title("Not enough permissions.")
        .description("You don't have enough permissions to use this command."))
      .wrap());
  }
  let config: Option<ServerConfig> = ::bot::CONNECTION.with(|c| {
    use database::schema::server_configs::dsl;
    dsl::server_configs
      .filter(dsl::server_id.eq(guild.read().id.to_u64()))
      .first(c)
      .optional()
      .chain_err(|| "could not load channel configs")
  })?;
  if args.len() < 1 {
    let status = match config.and_then(|c| c.timeout_role) {
      Some(role) => role,
      None => String::from("unset (disabled)")
    };
    return Ok(format!("Timeout role on {}: {}", guild.read().name, status).into());
  }
  let role_name = args[0].to_lowercase();
  let role = match guild.read().roles.values().find(|r| r.name.to_lowercase() == role_name) {
    Some(r) => r.clone(),
    None => return Err(format!("No role by the name `{}`.", &args[2]).into())
  };
  match config {
    Some(mut conf) => {
      conf.timeout_role = Some(role.name.clone());
      ::bot::CONNECTION.with(|c| conf.save_changes::<ServerConfig>(c).chain_err(|| "could not update config"))?;
    },
    None => {
      ::bot::CONNECTION.with(|c| {
        let new = NewServerConfig {
          server_id: guild.read().id.into(),
          timeout_role: Some(role.name.clone())
        };
        diesel::insert_into(::database::schema::server_configs::table)
          .values(&new)
          .execute(c)
          .chain_err(|| "could not add config")
      })?;
    }
  }
  let guild = guild.read();
  Ok(format!("Set timeout role in {} to {}.", guild.name, role.name).into())
}
