use database::schema::*;
use database::models::U64;

insertable! {
  #[derive(Debug, Queryable, Identifiable)]
  pub struct Reaction,
  #[derive(Debug, Insertable)]
  #[table_name = "reactions"]
  pub struct NewReaction {
    pub server_id: U64,
    pub channel_id: U64,
    pub message_id: U64,
    pub emoji: String,
    pub role_id: U64
  }
}
