use database::schema::*;
use database::models::Message;

#[derive(Debug, Queryable, Identifiable, Associations)]
#[belongs_to(Message)]
pub struct Edit {
  pub id: i32,
  pub message_id: i32,
  pub content: String
}

#[derive(Debug, Insertable)]
#[table_name = "edits"]
pub struct NewEdit {
  pub message_id: i32,
  pub content: String
}
