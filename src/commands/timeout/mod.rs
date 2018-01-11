use error::*;
use database::models::ServerConfig;

use diesel::prelude::*;

use serenity::model::id::RoleId;
use serenity::model::guild::Guild;
use serenity::model::permissions::Permissions;
use serenity::model::channel::{PermissionOverwrite, PermissionOverwriteType};

pub mod timeout_command;
pub mod untimeout;

pub use self::timeout_command::TimeoutCommand;
pub use self::untimeout::UntimeoutCommand;

lazy_static! {
  static ref ROLE_PERMISSIONS: Permissions = {
    let mut perm = Permissions::empty();
    perm.insert(Permissions::READ_MESSAGES);
    perm.insert(Permissions::READ_MESSAGE_HISTORY);
    perm.insert(Permissions::CONNECT);
    perm
  };
  static ref DENY_PERMISSIONS: Permissions = {
    let mut perm = Permissions::all();
    perm.remove(Permissions::READ_MESSAGES);
    perm.remove(Permissions::READ_MESSAGE_HISTORY);
    perm.remove(Permissions::CONNECT);
    perm
  };
}

pub fn set_up_timeouts(guild: &Guild) -> Result<RoleId> {
  let server_config: Option<ServerConfig> = ::bot::CONNECTION.with(|c| {
    use database::schema::server_configs::dsl;
    dsl::server_configs
      .filter(dsl::server_id.eq(guild.id.0.to_string()))
      .first(c)
      .optional()
      .chain_err(|| "could not load server configs")
  })?;
  let role_name = match server_config.and_then(|c| c.timeout_role) {
    Some(ref r) => r.to_string(),
    None => return Err("no timed-out role name".into())
  };
  let lower = role_name.to_lowercase();

  let role_id = match guild.roles.values().find(|r| r.name.to_lowercase() == lower) {
    Some(r) => r.id,
    None =>  guild.create_role(|e| e
      .name(&role_name)
      .permissions(*ROLE_PERMISSIONS))
      .chain_err(|| "could not create role")?
      .id
  };

  let target = PermissionOverwrite {
    kind: PermissionOverwriteType::Role(role_id),
    allow: Permissions::empty(),
    deny: *DENY_PERMISSIONS,
  };

  for channel in guild.channels.values() {
    if channel.read().permission_overwrites.iter().any(|o| o.kind == target.kind) {
      continue;
    }
    if let Err(e) = channel.read().create_permission(&target) {
      warn!("could not create permission overwrite for {}: {}", channel.read().id, e);
    }
  }
  Ok(role_id)
}
