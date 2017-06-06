pub mod tag_command;
pub mod autotag;
pub mod update_tags;

pub use self::tag_command::TagCommand;
pub use self::autotag::AutoTagCommand;
pub use self::update_tags::UpdateTagsCommand;

use LalafellBot;
use database::AutotagUser;

use xivdb::error::*;

use discord::model::{UserId, LiveServer, Role, RoleId};

use std::sync::{Arc, Mutex};

lazy_static! {
  // Limbo roles are roles that may or may not be added to the Discord bot state.
  static ref LIMBO_ROLES: Mutex<Vec<Role>> = Mutex::default();
}

pub struct Tagger;

impl Tagger {
  pub fn search_tag(bot: Arc<LalafellBot>, who: UserId, on: &LiveServer, server: &str, character_name: &str, ignore_verified: bool) -> Result<Option<String>> {
    let params = &[
      ("one", "characters"),
      ("strict", "on"),
      ("server|et", server)
    ];

    let res = bot.xivdb.search(character_name, params).chain_err(|| "could not query XIVDB")?;

    let search_chars = res.characters.unwrap().results;
    if search_chars.is_empty() {
      return Ok(Some(format!("Could not find any character by the name {} on {}.", character_name, server)));
    }

    let char_id = match search_chars[0]["id"].as_u64() {
      Some(u) => u,
      None => return Err("character ID was not a u64".into())
    };

    let name = match search_chars[0]["name"].as_str() {
      Some(s) => s,
      None => return Err("character name was not a string".into())
    };

    if name.to_lowercase() != character_name.to_lowercase() {
      return Ok(Some(format!("Could not find any character by the name {} on {}.", character_name, server)));
    }

    Tagger::tag(bot, who, on, char_id, ignore_verified)
  }

  pub fn tag(bot: Arc<LalafellBot>, who: UserId, on: &LiveServer, char_id: u64, ignore_verified: bool) -> Result<Option<String>> {
    let is_verified = match bot.database.read().unwrap().autotags.users.iter().find(|u| u.user_id == who.0 && u.server_id == on.id.0) {
      Some(u) => {
        if u.verification.verified && !ignore_verified && char_id != u.character_id {
          return Ok(Some(format!("{} is verified as {} on {}, so they cannot switch to another account.", who.mention(), u.character, u.server)));
        }
        u.verification.verified
      },
      None => false
    };

    let member = bot.discord.get_member(on.id, who).chain_err(|| "could not get member for tagging")?;

    let character = bot.xivdb.character(char_id).chain_err(|| "could not look up character")?;

    bot.database.write().unwrap().autotags.update_or_add(AutotagUser::new(
      who.0,
      on.id.0,
      character.lodestone_id,
      &character.name,
      &character.server
    ));

    // Get a copy of the roles on the server.
    let mut roles = on.roles.clone();
    // Check for existing limbo roles.
    {
      let limbo = &mut *LIMBO_ROLES.lock().unwrap();
      for role in &roles {
        // If the server has updated to contain the limbo role, remove it.
        if let Some(i) = limbo.iter().position(|x| x.id == role.id) {
          limbo.remove(i);
        }
      }
    }
    // Get a copy of the limbo roles.
    let limbo = LIMBO_ROLES.lock().unwrap().clone();
    // Extend the server roles with the limbo roles.
    roles.extend(limbo);
    let mut created_roles = Vec::new();
    let mut add_roles = Vec::new();
    match roles.iter().find(|x| x.name.to_lowercase() == character.data.race.to_lowercase()) {
      Some(r) => add_roles.push(r),
      None => {
        let role = bot.discord.create_role(on.id, Some(&character.data.race.to_lowercase()), None, None, None, None).chain_err(|| "could not create role")?;
        created_roles.push(role);
      }
    }
    match roles.iter().find(|x| x.name.to_lowercase() == character.data.gender.to_lowercase()) {
      Some(r) => add_roles.push(r),
      None => {
        let role = bot.discord.create_role(on.id, Some(&character.data.gender.to_lowercase()), None, None, None, None).chain_err(|| "could not create role")?;
        created_roles.push(role);
      }
    }
    match roles.iter().find(|x| x.name.to_lowercase() == character.server.to_lowercase()) {
      Some(r) => add_roles.push(r),
      None => {
        let role = bot.discord.create_role(on.id, Some(&character.server.to_lowercase()), None, None, None, None).chain_err(|| "could not create role")?;
        created_roles.push(role);
      }
    }
    if is_verified {
      match roles.iter().find(|x| x.name.to_lowercase() == "verified") {
        Some(r) => add_roles.push(r),
        None => {
          let role = bot.discord.create_role(on.id, Some("Verified"), None, None, None, None).chain_err(|| "could not create role")?;
          created_roles.push(role);
        }
      }
    }

    // If we created any roles, the server may or may not update with them fast enough, so store a copy in the limbo
    // roles.
    {
      let mut limbo = &mut *LIMBO_ROLES.lock().unwrap();
      for created in &created_roles {
        limbo.push(created.clone());
      }
    }

    debug!("Created the following roles:\n:{:#?}", created_roles);

    debug!("Existing roles:\n{:#?}", add_roles);

    // Extend the roles to add with the roles we created.
    add_roles.extend(created_roles.iter());
    // Get all the roles that are part of groups
    let all_group_roles: Vec<String> = bot.config.roles.groups.iter().flat_map(|x| x).map(|x| x.to_lowercase()).collect();
    // Filter all roles on the server to only the roles the member has
    let keep: Vec<&Role> = roles.iter().filter(|x| member.roles.contains(&x.id)).collect();
    // Filter all the roles the member has, keeping the ones not in a group. These roles will not be touched when
    // updating the tag.
    let keep: Vec<&Role> = keep.into_iter().filter(|x| !all_group_roles.contains(&x.name.to_lowercase())).collect();
    debug!("Roles to keep:\n{:#?}", keep);
    // Combine the two sets of roles and map them to IDs
    let mut role_set: Vec<RoleId> = add_roles.iter().map(|r| r.id).chain(keep.into_iter().map(|r| r.id)).collect();
    // Sort the IDs so we can dedup them
    role_set.sort();
    // Remove the duplicate roles, if any
    role_set.dedup();

    debug!("Final role set:\n{:#?}", role_set);

    // Only update the roles if they are different
    let different = {
      use std::collections::HashSet;
      let member_roles: HashSet<u64> = member.roles.iter().map(|x| x.0).collect();
      let actual_role_set: HashSet<u64> = role_set.iter().map(|x| x.0).collect();
      member_roles != actual_role_set
    };
    if different {
      bot.discord.edit_member_roles(on.id, who, &role_set).chain_err(|| "could not add roles")?;
    }

    // cannot edit nickname of those with a higher role
    let nick = match member.nick {
      Some(n) => n,
      None => Default::default()
    };
    if nick != character.name {
      bot.discord.edit_member(on.id, who, |e| e.nickname(&character.name)).ok();
    }
    Ok(None)
  }
}
