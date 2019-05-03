use crate::database::schema::*;

#[derive(Debug, Queryable)]
pub struct Role {
  pub role_name: String,
}

#[derive(Debug, Insertable)]
#[table_name = "roles"]
pub struct NewRole {
  pub role_name: String,
}
