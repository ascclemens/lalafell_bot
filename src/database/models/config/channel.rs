use crate::database::schema::*;
use crate::database::models::U64;

insertable! {
  #[derive(Debug, Queryable, Identifiable, AsChangeset)]
  pub struct ChannelConfig,
  #[derive(Debug, Insertable)]
  #[table_name = "channel_configs"]
  pub struct NewChannelConfig {
    pub server_id: U64,
    pub channel_id: U64,
    pub image_dump_allowed: Option<bool>
  }
}
