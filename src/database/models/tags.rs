use database::schema::*;
use database::models::U64;

use chrono::Utc;

#[derive(Debug, Queryable, Identifiable)]
pub struct Tag {
  pub id: i32,
  pub user_id: U64,
  pub server_id: U64,
  pub character_id: U64,
  pub character: String,
  pub server: String,
  pub last_updated: i64
}

#[derive(Debug, Insertable)]
#[table_name = "tags"]
pub struct NewTag {
  pub user_id: String,
  pub server_id: String,
  pub character_id: String,
  pub character: String,
  pub server: String,
  pub last_updated: i64
}

impl NewTag {
  pub fn new(user_id: u64, server_id: u64, character_id: u64, character: &str, server: &str) -> Self {
    NewTag {
      user_id: user_id.to_string(),
      server_id: server_id.to_string(),
      character_id: character_id.to_string(),
      character: character.to_owned(),
      server: server.to_owned(),
      last_updated: Utc::now().timestamp()
    }
  }
}
