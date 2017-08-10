use database::models::{ServerConfig, NewServerConfig};

use discord::model::{UserId, LiveServer};

use diesel;
use diesel::prelude::*;

use lalafell::error::*;
use lalafell::commands::prelude::*;

pub fn timeout_role<'a>(author: UserId, server: &LiveServer, args: &[String]) -> CommandResult<'a> {
  if author != server.owner_id {
    return Err(ExternalCommandFailure::default()
      .message(|e: EmbedBuilder| e
        .title("Not enough permissions.")
        .description("You don't have enough permissions to use this command."))
      .wrap());
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
    Some(mut conf) => {
      conf.timeout_role = Some(role.name.clone());
      ::bot::CONNECTION.with(|c| conf.save_changes::<ServerConfig>(c).chain_err(|| "could not update config"))?;
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
