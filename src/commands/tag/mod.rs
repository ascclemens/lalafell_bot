pub mod autotag;
pub mod tag_command;
pub mod update_tag;
pub mod update_tags;

pub use self::autotag::AutoTagCommand;
pub use self::tag_command::TagCommand;
pub use self::update_tag::UpdateTagCommand;
pub use self::update_tags::UpdateTagsCommand;

use bot::BotEnv;
use database::models::{ToU64, Tag, NewTag, Verification, Role as DbRole};

use lalafell::error::*;
use lalafell::commands::prelude::*;

use serenity::Error as SError;
use serenity::builder::EditRole;
use serenity::model::guild::Role;
use serenity::prelude::Mentionable;
use serenity::model::id::{RoleId, UserId};
use serenity::http::{HttpError, StatusCode};

use diesel::prelude::*;

use url::Url;

use hyper_rustls::HttpsConnector;
use make_hyper_great_again::Client;

use std::thread;
use std::sync::Mutex;
use std::collections::HashSet;

lazy_static! {
  // Limbo roles are roles that may or may not be added to the Discord bot state.
  static ref LIMBO_ROLES: Mutex<Vec<Role>> = Mutex::default();
}

pub struct Tagger;

impl Tagger {
  fn add_to_xivdb<N: Into<String>, S: Into<String>>(name: N, server: S) {
    let name: String = name.into();
    let server: String = server.into();
    thread::spawn(move || {
      let mut url = Url::parse("https://xivsync.com/character/search").unwrap();
      url.query_pairs_mut()
        .append_pair("name", &name)
        .append_pair("server", &server);
      let client = Client::create_connector(|c| HttpsConnector::new(4, &c.handle())).unwrap();
      client.get(url).send().ok();
    });
  }

  pub fn search_tag(env: &BotEnv, who: UserId, on: GuildId, server: &str, character_name: &str, ignore_verified: bool) -> Result<Option<String>> {
    let params = &[
      ("one", "characters"),
      ("strict", "on"),
      ("server|et", server)
    ];

    let res = env.xivdb.search(character_name, params).chain_err(|| "could not query XIVDB")?;

    let search_chars = res.characters.unwrap().results;
    if search_chars.is_empty() {
      Tagger::add_to_xivdb(character_name, server);
      return Ok(Some(format!("Could not find any character by the name {} on {} in the XIVDB database.\nIf you typed everything correctly, please wait five minutes for your character to get added to the database, then try again.", character_name, server)));
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

    Tagger::tag(env, who, on, char_id, ignore_verified)
  }

  fn find_or_create_role(guild: GuildId, name: &str, add_roles: &mut Vec<Role>, created_roles: &mut Vec<Role>) -> Result<()> {
    Tagger::find_or_create_role_and(guild, name, add_roles, created_roles, |r| r)
  }

  fn find_or_create_role_and<F>(guild: GuildId, name: &str, add_roles: &mut Vec<Role>, created_roles: &mut Vec<Role>, f: F) -> Result<()>
    where F: FnOnce(EditRole) -> EditRole
  {
    let lower_name = name.to_lowercase();
    let guild = guild.get().chain_err(|| "could not get guild")?;
    match guild.roles.values().find(|x| x.name.to_lowercase() == lower_name) {
      Some(r) => add_roles.push(r.clone()),
      None => {
        let role = guild.create_role(|r| f(r).name(&lower_name)).chain_err(|| "could not create role")?;
        created_roles.push(role);
      }
    }
    Ok(())
  }

  fn get_roles() -> Result<Vec<String>> {
    let roles: Vec<DbRole> = ::bot::CONNECTION.with(|c| {
      use database::schema::roles::dsl;
      dsl::roles.load(c).chain_err(|| "could not load roles")
    })?;
    Ok(roles.into_iter().map(|x| x.role_name.to_lowercase()).collect())
  }

  pub fn tag(env: &BotEnv, who: UserId, on: GuildId, char_id: u64, ignore_verified: bool) -> Result<Option<String>> {
    let tag: Option<Tag> = ::bot::CONNECTION.with(|c| {
      use database::schema::tags::dsl;
      dsl::tags
        .filter(dsl::user_id.eq(who.to_u64()).and(dsl::server_id.eq(on.to_u64())))
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
    let member = match on.member(who) {
      Ok(m) => m,
      Err(SError::Http(HttpError::UnsuccessfulRequest(ref r))) if r.status == StatusCode::NotFound => {
        ::bot::CONNECTION.with(|c| {
          use database::schema::tags::dsl;
          ::diesel::delete(dsl::tags
            .filter(dsl::user_id.eq(who.to_u64()).and(dsl::server_id.eq(on.to_u64()))))
            .execute(c)
            .chain_err(|| format!("could not remove tag {} on {}, but it was expected to be in database", who.0, on.0))
        })?;
        bail!("could not get user {} as a member of {}: removing their tag", who.0, on.0);
      },
      Err(e) => return Err(e).chain_err(|| "could not get member for tagging")
    };

    let character = env.xivdb.character(char_id).chain_err(|| "could not look up character")?;

    let tag: Option<Tag> = ::bot::CONNECTION.with(|c| {
      use database::schema::tags::dsl;
      dsl::tags
        .filter(dsl::user_id.eq(who.to_u64()).and(dsl::server_id.eq(on.to_u64())))
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
          on.0,
          character.lodestone_id,
          &character.name,
          &character.server
        );
        ::bot::CONNECTION.with(|c| {
          use database::schema::tags;
          ::diesel::insert_into(tags::table).values(&new_tag).execute(c).chain_err(|| "could not insert tag")
        })?;
      }
    }

    // Get a copy of the roles on the server.
    let mut roles: Vec<Role> = match on.find() {
      Some(g) => g.read().roles.values().cloned().collect(),
      None => on.get().chain_err(|| "could not get guild")?.roles.values().cloned().collect()
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
    Tagger::find_or_create_role(on, &character.data.race, &mut add_roles, &mut created_roles)?;
    Tagger::find_or_create_role(on, &character.data.gender, &mut add_roles, &mut created_roles)?;
    Tagger::find_or_create_role_and(on, &character.server, &mut add_roles, &mut created_roles, |r| r.hoist(true))?;

    if is_verified {
      Tagger::find_or_create_role(on, "verified", &mut add_roles, &mut created_roles)?;
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
      on.edit_member(who, |m| m.roles(&role_set)).chain_err(|| "could not add roles")?;
    }

    // cannot edit nickname of those with a higher role
    if member.nick.as_ref() != Some(&character.name) {
      on.edit_member(who, |m| m.nickname(&character.name)).ok();
    }
    Ok(None)
  }
}

#[derive(Debug, Deserialize)]
struct DiscordNotFoundError {
  code: u64,
  message: Option<String>
}
