use database::schema::*;
use database::models::U64;

insertable! {
  #[derive(Debug, Queryable, Identifiable, AsChangeset)]
  #[table_name = "tag_queue"]
  pub struct TagQueue,
  #[derive(Debug, Insertable)]
  #[table_name = "tag_queue"]
  pub struct NewTagQueue {
    pub user_id: U64,
    pub server_id: U64,
    pub server: String,
    pub character: String
  }
}

impl NewTagQueue {
  pub fn new(user_id: u64, server_id: u64, server: &str, character: &str) -> Self {
    NewTagQueue {
      user_id: user_id.into(),
      server_id: server_id.into(),
      server: server.to_owned(),
      character: character.to_owned()
    }
  }
}
