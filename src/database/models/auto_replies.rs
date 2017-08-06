use database::schema::*;
use database::models::U64;

insertable! {
  #[derive(Debug, Queryable, Identifiable)]
  #[table_name = "auto_replies"]
  pub struct AutoReply,
  #[derive(Debug, Insertable)]
  #[table_name = "auto_replies"]
  pub struct NewAutoReply {
    pub server_id: U64,
    pub channel_id: U64,
    pub message: String,
    pub on_join: bool
  }
}
