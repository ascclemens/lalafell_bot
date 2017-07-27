use database::schema::*;
use database::models::U64;

insertable! {
  #[derive(Debug, Queryable, Identifiable)]
  pub struct Message,
  #[derive(Debug, Insertable)]
  #[table_name = "messages"]
  pub struct NewMessage {
    pub message_id: U64,
    pub channel_id: U64,
    pub content: String
  }
}
