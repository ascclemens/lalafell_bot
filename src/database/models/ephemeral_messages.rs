use database::schema::*;
use database::models::U64;

insertable! {
  #[derive(Debug, Queryable, Identifiable, AsChangeset)]
  pub struct EphemeralMessage,
  #[derive(Debug, Insertable)]
  #[table_name = "ephemeral_messages"]
  pub struct NewEphemeralMessage {
    pub guild_id: U64,
    pub channel_id: U64,
    pub message_id: U64,
    pub expires_on: i64
  }
}

impl NewEphemeralMessage {
  pub fn new(guild_id: u64, channel_id: u64, message_id: u64, expires_on: i64) -> Self {
    NewEphemeralMessage {
      guild_id: guild_id.into(),
      channel_id: channel_id.into(),
      message_id: message_id.into(),
      expires_on
    }
  }
}
