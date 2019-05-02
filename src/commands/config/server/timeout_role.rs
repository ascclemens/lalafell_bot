use crate::database::models::{ToU64, ServerConfig, NewServerConfig};

use serenity::builder::CreateEmbed;
use serenity::model::id::UserId;

use diesel;
use diesel::prelude::*;

use lalafell::error::*;
use lalafell::commands::prelude::*;

use unicase::UniCase;

pub struct TimeoutRoleCommand;

#[derive(Debug, StructOpt)]
pub struct Params {
  #[structopt(help = "The role to set as the timeout role")]
  #[structopt(raw(use_delimiter = "false"))]
  role: Option<String>
}

impl<'a> TimeoutRoleCommand {
  pub fn run(&self, ctx: &Context, author: UserId, guild: GuildId, params: Params) -> CommandResult<'a> {
    let guild = guild.to_guild_cached(&ctx).chain_err(|| "could not find guild")?;
    if author != guild.read().owner_id {
      return Err(ExternalCommandFailure::default()
        .message(|e: &mut CreateEmbed| e
          .title("Not enough permissions.")
          .description("You don't have enough permissions to use this command."))
        .wrap());
    }
    let config: Option<ServerConfig> = crate::bot::with_connection(|c| {
      use crate::database::schema::server_configs::dsl;
      dsl::server_configs
        .filter(dsl::server_id.eq(guild.read().id.to_u64()))
        .first(c)
        .optional()
    }).chain_err(|| "could not load channel configs")?;
    match params.role {
      Some(given) => {
        let role_name = UniCase::new(&given);
        let role = match guild.read().roles.values().find(|r| UniCase::new(&r.name) == role_name) {
          Some(r) => r.clone(),
          None => return Err(format!("No role by the name `{}`.", given).into())
        };
        match config {
          Some(mut conf) => {
            conf.timeout_role = Some(role.name.clone());
            crate::bot::with_connection(|c| conf.save_changes::<ServerConfig>(c)).chain_err(|| "could not update config")?;
          },
          None => {
            crate::bot::with_connection(|c| {
              let new = NewServerConfig {
                server_id: guild.read().id.into(),
                timeout_role: Some(role.name.clone())
              };
              diesel::insert_into(crate::database::schema::server_configs::table)
                .values(&new)
                .execute(c)
            }).chain_err(|| "could not add config")?;
          }
        }
        let guild = guild.read();
        Ok(format!("Set timeout role in {} to {}.", guild.name, role.name).into())
      },
      None => {
        let status = match config.and_then(|c| c.timeout_role) {
          Some(role) => role,
          None => String::from("unset (disabled)")
        };
        Ok(format!("Timeout role on {}: {}", guild.read().name, status).into())
      }
    }
  }
}
