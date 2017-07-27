use database::schema::*;
use database::models::Message;

insertable! {
  #[derive(Debug, Queryable, Identifiable, Associations)]
  #[belongs_to(Message)]
  pub struct Edit,
  #[derive(Debug, Insertable)]
  #[table_name = "edits"]
  pub struct NewEdit {
    pub message_id: i32,
    pub content: String
  }
}
