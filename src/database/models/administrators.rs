use crate::database::schema::*;
use crate::database::models::U64;

#[derive(Debug, Queryable)]
pub struct Administrator {
  pub user_id: U64
}
#[derive(Debug, Insertable)]
#[table_name = "administrators"]
pub struct NewAdministrator {
  pub user_id: U64
}
