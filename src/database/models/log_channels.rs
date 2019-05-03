use crate::database::{
  schema::*,
  models::U64,
};

#[derive(Debug, Queryable)]
pub struct LogChannel {
  pub server_id: U64,
  pub channel_id: U64,
}

#[derive(Debug, Insertable)]
#[table_name = "log_channels"]
pub struct NewLogChannel {
  pub server_id: U64,
  pub channel_id: U64,
}
