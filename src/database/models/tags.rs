use database::schema::*;

#[derive(Debug, Queryable, Identifiable)]
pub struct Tag {
  pub id: i32,
  pub user_id: f64,
  pub server_id: f64,
  pub character_id: f64,
  pub character: String,
  pub server: String,
  pub last_updated: i32
}
