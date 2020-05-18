pub mod autotag;
pub mod queue_tag;
pub mod tag_command;
pub mod update_tag;
pub mod update_tags;

pub use self::{
  autotag::AutoTagCommand,
  queue_tag::QueueTagCommand,
  tag_command::TagCommand,
  update_tag::UpdateTagCommand,
  update_tags::UpdateTagsCommand,
};

use crate::{
  bot::BotEnv,
  database::models::{ToU64, Tag, NewTag, U64, Verification, Role as DbRole},
};

use diesel::prelude::*;

use failure::Fail;

use ffxiv::{World, Race};

use lalafell::error::*;
use lalafell::commands::prelude::*;

use serenity::{
  Error as SError,
  builder::EditRole,
  model::{
    guild::Role,
    permissions::Permissions,
  },
  prelude::Mentionable,
  model::id::{RoleId, UserId},
  http::{HttpError, StatusCode},
};

use unicase::UniCase;

use lodestone_api_client::{
  prelude::*,
  models::{
    RouteResult,
    character::Gender,
  },
};

use std::{
  collections::HashSet,
  sync::Mutex,
};

lazy_static! {
  // Limbo roles are roles that may or may not be added to the Discord bot state.
  static ref LIMBO_ROLES: Mutex<Vec<Role>> = Mutex::default();
}

pub struct Tagger;

impl Tagger {
  pub fn search_tag(env: &BotEnv, who: UserId, on: GuildId, world: World, character_name: &str, force: bool) -> Result<Option<String>> {
    let res = env.lodestone
      .character_search()
      .name(character_name)
      .world(world)
      .send()
      .map_err(Fail::compat)
      .chain_err(|| "could not query Lodestone API")?;

    let res = match res {
      RouteResult::Cached { result, .. } | RouteResult::Success { result, .. } | RouteResult::Scraped { result } => result,
      _ => return Ok(Some("An error occurred. Try again later.".into())),
    };

    let uni_char_name = UniCase::new(character_name);
    let character = match res.results.into_iter().find(|x| UniCase::new(&x.name) == uni_char_name) {
      Some(c) => c,
      None => return Ok(Some(format!("Could not find any character by the name {} on {} on the Lodestone.", character_name, world))),
    };
    if UniCase::new(character.name) != UniCase::new(character_name) {
      return Ok(Some(format!("Could not find any character by the name {} on {}.", character_name, world)));
    }

    Tagger::tag(env, who, on, character.id as u64, force, true)
  }

  fn find_or_create_role(env: &BotEnv, guild: GuildId, name: &str, add_roles: &mut Vec<Role>, created_roles: &mut Vec<Role>) -> Result<()> {
    Tagger::find_or_create_role_and(env, guild, name, add_roles, created_roles, |r| r)
  }

  fn find_or_create_role_and<F>(env: &BotEnv, guild: GuildId, name: &str, add_roles: &mut Vec<Role>, created_roles: &mut Vec<Role>, f: F) -> Result<()>
    where F: FnOnce(&mut EditRole) -> &mut EditRole,
  {
    let uni_name = UniCase::new(name);
    let guild = guild.to_partial_guild(env.http()).chain_err(|| "could not get guild")?;
    match guild.roles.values().find(|x| UniCase::new(&x.name) == uni_name) {
      Some(r) => add_roles.push(r.clone()),
      None => {
        let role = guild.create_role(env.http(), |r| f(r.permissions(Permissions::empty())).name(&name.to_lowercase())).chain_err(|| "could not create role")?;
        created_roles.push(role);
      }
    }
    Ok(())
  }

  fn get_roles() -> Result<Vec<String>> {
    let roles: Vec<DbRole> = crate::bot::with_connection(|c| {
      use crate::database::schema::roles::dsl;
      dsl::roles.load(c)
    }).chain_err(|| "could not load roles")?;
    Ok(roles.into_iter().map(|x| x.role_name.to_lowercase()).collect())
  }

  pub fn tag(env: &BotEnv, who: UserId, on: GuildId, char_id: u64, force: bool, wait: bool) -> Result<Option<String>> {
    // Trolls always make sure we can't have nice things.
    let existing_tags: i64 = crate::bot::with_connection(|c| {
      use crate::database::schema::tags::dsl;
      dsl::tags
        .select(diesel::dsl::count(dsl::id))
        .filter(dsl::character_id.eq(U64::from(char_id))
          .and(dsl::server_id.eq(on.to_u64()))
          .and(dsl::user_id.ne(who.to_u64())))
        .first(c)
    }).chain_err(|| "could not load tags")?;
    if !force && existing_tags > 0 {
      return Ok(Some("Someone is already tagged as that character.\n\nIf this is an alternate account, please let the mods know.".into()));
    }

    let tag: Option<Tag> = crate::bot::with_connection(|c| {
      use crate::database::schema::tags::dsl;
      dsl::tags
        .filter(dsl::user_id.eq(who.to_u64()).and(dsl::server_id.eq(on.to_u64())))
        .first(c)
        .optional()
    }).chain_err(|| "could not load tags")?;
    let is_verified = match tag {
      Some(t) => {
        let verification: Verification = crate::bot::with_connection(|c| {
          Verification::belonging_to(&t).first(c).optional()
        }).chain_err(|| "could not load verifications")?.unwrap_or_default();
        if verification.verified && !force && char_id != *t.character_id {
          return Ok(Some(format!("{} is verified as {} on {}, so they cannot switch to another account.", who.mention(), t.character, t.server)));
        }
        verification.verified
      },
      None => false
    };

    // This is still a disaster, just slightly less so
    let member = match env.cache().read().member(&on, &who) {
      Some(m) => Ok(m),
      None => env.http().get_member(on.0, who.0),
    };
    let member = match member {
      Ok(m) => m,
      Err(SError::Http(box HttpError::UnsuccessfulRequest(ref r))) if r.status_code == StatusCode::NOT_FOUND => {
        crate::bot::with_connection(|c| {
          use crate::database::schema::tags::dsl;
          diesel::delete(dsl::tags
            .filter(dsl::user_id.eq(who.to_u64()).and(dsl::server_id.eq(on.to_u64()))))
            .execute(c)
        }).chain_err(|| format!("could not remove tag {} on {}, but it was expected to be in database", who.0, on.0))?;
        bail!("could not get user {} as a member of {}: removing their tag", who.0, on.0);
      },
      Err(e) => return Err(e).chain_err(|| "could not get member for tagging")
    };

    let res = env.lodestone
      .character(char_id.into())
      .send()
      .map_err(Fail::compat)
      .chain_err(|| "could not look up character")?;

    let character = match res {
      RouteResult::Cached { result, .. } | RouteResult::Success { result, .. } | RouteResult::Scraped { result } => result,
      RouteResult::Adding { .. } if wait => {
        std::thread::sleep(std::time::Duration::from_secs(5));
        return Tagger::tag(env, who, on, char_id, force, false);
      },
      RouteResult::Adding { .. } => return Ok(Some("That character is now being added to the database. Try again in one minute.".into())),
      RouteResult::NotFound => return Ok(Some("No such character.".into())),
      RouteResult::Error { error } => return Ok(Some(format!("An error occurred: `{}`. Try again later.", error))),
    };

    let tag: Option<Tag> = crate::bot::with_connection(|c| {
      use crate::database::schema::tags::dsl;
      dsl::tags
        .filter(dsl::user_id.eq(who.to_u64()).and(dsl::server_id.eq(on.to_u64())))
        .first(c)
        .optional()
    }).chain_err(|| "could not load tags")?;
    match tag {
      Some(mut t) => {
        let (id, name, server) = (character.id, character.name.clone(), character.world);
        t.character_id = id.into();
        t.character = name;
        t.server = server.to_string();
        crate::bot::with_connection(|c| t.save_changes::<Tag>(c)).chain_err(|| "could not update tag")?;
      },
      None => {
        let new_tag = NewTag::new(
          who.0,
          on.0,
          character.id,
          &character.name,
          character.world.as_str()
        );
        crate::bot::with_connection(|c| {
          use crate::database::schema::tags;
          diesel::insert_into(tags::table).values(&new_tag).execute(c)
        }).chain_err(|| "could not insert tag")?;
      }
    }

    // Get a copy of the roles on the server.
    let mut roles: Vec<Role> = match on.to_guild_cached(env.cache_lock()) {
      Some(g) => g.read().roles.values().cloned().collect(),
      None => on.to_partial_guild(env.http()).chain_err(|| "could not get guild")?.roles.values().cloned().collect()
    };

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
    let race = match character.race {
      Race::AuRa => "au ra",
      Race::Elezen => "elezen",
      Race::Hrothgar => "hrothgar",
      Race::Hyur => "hyur",
      Race::Lalafell => "lalafell",
      Race::Miqote => "miqo'te",
      Race::Roegadyn => "roegadyn",
      Race::Viera => "viera",
    };
    let gender = match character.gender {
      Gender::Female => "female",
      Gender::Male => "male",
    };
    let data_centre = character.world.data_center();
    Tagger::find_or_create_role(env, on, race, &mut add_roles, &mut created_roles)?;
    Tagger::find_or_create_role(env, on, gender, &mut add_roles, &mut created_roles)?;
    Tagger::find_or_create_role_and(env, on, character.world.as_str(), &mut add_roles, &mut created_roles, |r| r.hoist(true))?;
    Tagger::find_or_create_role(env, on, data_centre.as_str(), &mut add_roles, &mut created_roles)?;

    if is_verified {
      Tagger::find_or_create_role(env, on, "verified", &mut add_roles, &mut created_roles)?;
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
    let all_group_roles = Tagger::get_roles()?;
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
      on.edit_member(env.http(), who, |m| m.roles(&role_set)).chain_err(|| "could not add roles")?;
    }

    // cannot edit nickname of those with a higher role
    if member.nick.as_ref() != Some(&character.name) {
      on.edit_member(env.http(), who, |m| m.nickname(&character.name)).ok();
    }
    Ok(None)
  }
}

#[derive(Debug, Deserialize)]
struct DiscordNotFoundError {
  code: u64,
  message: Option<String>
}
