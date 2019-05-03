use crate::database::{
  schema::*,
  models::U64,
};

use chrono::Utc;

insertable! {
  #[derive(Debug, Queryable, Identifiable, AsChangeset)]
  pub struct Tag,
  #[derive(Debug, Insertable)]
  #[table_name = "tags"]
  pub struct NewTag {
    pub user_id: U64,
    pub server_id: U64,
    pub character_id: U64,
    pub character: String,
    pub server: String,
    pub last_updated: i64,
  }
}

impl NewTag {
  pub fn new(user_id: u64, server_id: u64, character_id: u64, character: &str, server: &str) -> Self {
    NewTag {
      user_id: user_id.into(),
      server_id: server_id.into(),
      character_id: character_id.into(),
      character: character.to_owned(),
      server: server.to_owned(),
      last_updated: Utc::now().timestamp(),
    }
  }
}
