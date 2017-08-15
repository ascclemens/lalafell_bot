pub mod tag_command;
pub mod autotag;
pub mod update_tags;
pub mod update_tag;

pub use self::tag_command::TagCommand;
pub use self::autotag::AutoTagCommand;
pub use self::update_tags::UpdateTagsCommand;
pub use self::update_tag::UpdateTagCommand;

use bot::LalafellBot;
use database::models::{Tag, NewTag, Verification};

use lalafell::error::*;

use discord;
use discord::model::{UserId, LiveServer, Role, RoleId};

use diesel::prelude::*;

use serde_json;

use std::sync::Mutex;
use std::collections::HashSet;

lazy_static! {
  // Limbo roles are roles that may or may not be added to the Discord bot state.
  static ref LIMBO_ROLES: Mutex<Vec<Role>> = Mutex::default();
}

pub struct Tagger;

impl Tagger {
  pub fn search_tag(bot: &LalafellBot, who: UserId, on: &LiveServer, server: &str, character_name: &str, ignore_verified: bool) -> Result<Option<String>> {
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
      None => bail!("character ID was not a u64")
    };

    let name = match search_chars[0]["name"].as_str() {
      Some(s) => s,
      None => bail!("character name was not a string")
    };

    if name.to_lowercase() != character_name.to_lowercase() {
      return Ok(Some(format!("Could not find any character by the name {} on {}.", character_name, server)));
    }

    Tagger::tag(bot, who, on, char_id, ignore_verified)
  }

  fn find_or_create_role(bot: &LalafellBot, server: &LiveServer, name: &str, add_roles: &mut Vec<Role>, created_roles: &mut Vec<Role>) -> Result<()> {
    let lower_name = name.to_lowercase();
    match server.roles.iter().find(|x| x.name.to_lowercase() == lower_name) {
      Some(r) => add_roles.push(r.clone()),
      None => {
        let role = bot.discord.create_role(server.id, Some(&lower_name), None, None, None, None).chain_err(|| "could not create role")?;
        created_roles.push(role);
      }
    }
    Ok(())
  }

  pub fn tag(bot: &LalafellBot, who: UserId, on: &LiveServer, char_id: u64, ignore_verified: bool) -> Result<Option<String>> {
    let tag: Option<Tag> = ::bot::CONNECTION.with(|c| {
      use database::schema::tags::dsl;
      dsl::tags
        .filter(dsl::user_id.eq(who.0.to_string()).and(dsl::server_id.eq(on.id.0.to_string())))
        .first(c)
        .optional()
        .chain_err(|| "could not load tags")
    })?;
    let is_verified = match tag {
      Some(t) => {
        let verification: Verification = ::bot::CONNECTION.with(|c| {
          Verification::belonging_to(&t).first(c).optional().chain_err(|| "could not load verifications")
        })?.unwrap_or_default();
        if verification.verified && !ignore_verified && char_id != *t.character_id {
          return Ok(Some(format!("{} is verified as {} on {}, so they cannot switch to another account.", who.mention(), t.character, t.server)));
        }
        verification.verified
      },
      None => false
    };

    // This is still a disaster, just slightly less so
    let member = match bot.discord.get_member(on.id, who) {
      Ok(m) => m,
      Err(discord::Error::Status(_, Some(discord_error))) => {
        let error: DiscordNotFoundError = serde_json::from_value(discord_error)
          .chain_err(|| "could not get member for tagging and could not parse error")?;
        let (remove, message) = match error.code {
          // Unknown user
          10013 => (true, format!("could not find user {}: removing from database", who.0)),
          // User not a member on this server
          10007 => (true, format!("user {} is not on server {}: removing from database", who.0, on.id.0)),
          _ => {
            let message = error.message.map(|x| format!(" ({})", x)).unwrap_or_default();
            let error_message = format!("could not get member for tagging with unknown error code: {}{}", error.code, message);
            (false, error_message)
          }
        };
        if remove {
          ::bot::CONNECTION.with(|c| {
            use database::schema::tags::dsl;
            ::diesel::delete(dsl::tags
              .filter(dsl::user_id.eq(who.0.to_string()).and(dsl::server_id.eq(on.id.0.to_string()))))
              .execute(c)
              .chain_err(|| format!("could not remove tag {} on {}, but it was expected to be in database", who.0, on.id.0))
          })?;
        }
        bail!(message);
      },
      Err(e) => return Err(e).chain_err(|| "could not get member for tagging")
    };

    let character = bot.xivdb.character(char_id).chain_err(|| "could not look up character")?;

    let tag: Option<Tag> = ::bot::CONNECTION.with(|c| {
      use database::schema::tags::dsl;
      dsl::tags
        .filter(dsl::user_id.eq(who.0.to_string()).and(dsl::server_id.eq(on.id.0.to_string())))
        .first(c)
        .optional()
        .chain_err(|| "could not load tags")
    })?;
    match tag {
      Some(mut t) => {
        let (id, name, server) = (character.lodestone_id, character.name.clone(), character.server.clone());
        t.character_id = id.into();
        t.character = name;
        t.server = server;
        ::bot::CONNECTION.with(|c| t.save_changes::<Tag>(c).chain_err(|| "could not update tag"))?;
      },
      None => {
        let new_tag = NewTag::new(
          who.0,
          on.id.0,
          character.lodestone_id,
          &character.name,
          &character.server
        );
        ::bot::CONNECTION.with(|c| {
          use database::schema::tags;
          ::diesel::insert(&new_tag).into(tags::table).execute(c).chain_err(|| "could not insert tag")
        })?;
      }
    }

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

    // Find or create the necessary roles
    let mut created_roles = Vec::new();
    let mut add_roles = Vec::new();
    Tagger::find_or_create_role(bot, on, &character.data.race, &mut add_roles, &mut created_roles)?;
    Tagger::find_or_create_role(bot, on, &character.data.gender, &mut add_roles, &mut created_roles)?;
    Tagger::find_or_create_role(bot, on, &character.server, &mut add_roles, &mut created_roles)?;

    if is_verified {
      Tagger::find_or_create_role(bot, on, "verified", &mut add_roles, &mut created_roles)?;
    }

    // If we created any roles, the server may or may not update with them fast enough, so store a copy in the limbo
    // roles.
    {
      let limbo = &mut *LIMBO_ROLES.lock().unwrap();
      for created in &created_roles {
        limbo.push(created.clone());
      }
    }

    debug!("Created the following roles:\n:{:#?}", created_roles);

    debug!("Existing roles:\n{:#?}", add_roles);

    // Extend the roles to add with the roles we created.
    add_roles.extend(created_roles);
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

#[derive(Debug, Deserialize)]
struct DiscordNotFoundError {
  code: u64,
  message: Option<String>
}
