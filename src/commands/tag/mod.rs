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

use std::sync::Arc;
use std::collections::HashMap;

pub struct Tagger;

impl Tagger {
  pub fn search_tag(bot: Arc<LalafellBot>, who: UserId, on: &LiveServer, server: &str, character_name: &str, ignore_verified: bool) -> Result<Option<String>> {
    let mut params = HashMap::new();
    params.insert(String::from("one"), String::from("characters"));
    params.insert(String::from("strict"), String::from("on"));
    params.insert(String::from("server|et"), server.to_string());

    let res = bot.xivdb.search(character_name.to_string(), params).chain_err(|| "could not query XIVDB")?;

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
    let is_verified = match bot.database.lock().unwrap().autotags.users.iter().find(|u| u.user_id == who.0 && u.server_id == on.id.0) {
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

    bot.database.lock().unwrap().autotags.update_or_remove(AutotagUser::new(
      who.0,
      on.id.0,
      character.lodestone_id,
      &character.name,
      &character.server
    ));

    let roles = &on.roles;
    let mut add_roles = Vec::new();
    if let Some(r) = roles.iter().find(|x| x.name.to_lowercase() == character.data.race.to_lowercase()) {
      add_roles.push(r);
    }
    if let Some(r) = roles.iter().find(|x| x.name.to_lowercase() == character.data.gender.to_lowercase()) {
      add_roles.push(r);
    }
    if let Some(r) = roles.iter().find(|x| x.name.to_lowercase() == character.server.to_lowercase()) {
      add_roles.push(r);
    }
    if is_verified {
      if let Some(r) = roles.iter().find(|x| x.name.to_lowercase() == "verified") {
        add_roles.push(r);
      }
    }

    let all_group_roles: Vec<&String> = bot.config.roles.groups.iter().flat_map(|x| x).collect();
    let keep: Vec<&Role> = roles.iter().filter(|x| member.roles.contains(&x.id)).collect();
    let keep: Vec<&Role> = keep.into_iter().filter(|x| !all_group_roles.contains(&&x.name)).collect();
    let mut role_set: Vec<RoleId> = add_roles.iter().map(|r| r.id).chain(keep.into_iter().map(|r| r.id)).collect();
    role_set.sort();
    role_set.dedup();

    if !role_set.iter().all(|r| member.roles.contains(r)) {
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
