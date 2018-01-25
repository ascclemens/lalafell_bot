use database::schema::*;
use database::models::U64;

#[derive(Debug, Queryable)]
pub struct PartyFinderConfig {
  pub server_id: U64,
  pub channel_id: U64,
  pub message_id: U64,
  pub role_id: U64,
  pub emoji: String,
  pub timeout: i64
}

#[derive(Debug, Insertable)]
#[table_name = "party_finder_configs"]
pub struct NewPartyFinderConfig {
  pub server_id: U64,
  pub channel_id: U64,
  pub message_id: U64,
  pub role_id: U64,
  pub emoji: String,
  pub timeout: i64
}
