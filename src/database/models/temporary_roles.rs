use database::schema::*;
use database::models::U64;

insertable! {
  #[derive(Debug, Queryable, Identifiable, AsChangeset)]
  pub struct TemporaryRole,
  #[derive(Debug, Insertable)]
  #[table_name = "temporary_roles"]
  pub struct NewTemporaryRole {
    pub guild_id: U64,
    pub user_id: U64,
    pub role_id: U64,
    pub message_id: U64,
    pub channel_id: Option<i64>,
    pub messages: Option<i32>,
    pub expires_on: Option<i64>
  }
}

impl NewTemporaryRole {
  pub fn new(guild_id: u64, user_id: u64, role_id: u64, message_id: u64, channel_id: Option<u64>, messages: Option<i32>, expires_on: Option<i64>) -> Self {
    NewTemporaryRole {
      guild_id: guild_id.into(),
      user_id: user_id.into(),
      role_id: role_id.into(),
      message_id: message_id.into(),
      channel_id: channel_id.map(|x| x as i64),
      messages,
      expires_on
    }
  }
}
