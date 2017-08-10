use database::schema::*;
use database::models::U64;

insertable! {
  #[derive(Debug, Queryable, Identifiable, AsChangeset)]
  pub struct ServerConfig,
  #[derive(Debug, Insertable)]
  #[table_name = "server_configs"]
  pub struct NewServerConfig {
    pub server_id: U64,
    pub timeout_role: Option<String>
  }
}
