use database::schema::*;

#[derive(Debug, Queryable, Identifiable)]
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
