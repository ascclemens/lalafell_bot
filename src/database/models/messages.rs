use database::schema::*;
use database::models::U64;

#[derive(Debug, Queryable, Identifiable)]
pub struct Message {
  pub id: i32,
  pub message_id: U64,
  pub channel_id: U64,
  pub content: String
}

#[derive(Debug, Insertable)]
#[table_name = "messages"]
pub struct NewMessage {
  pub message_id: String,
  pub channel_id: String,
  pub content: String
}
