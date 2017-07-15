use database::schema::*;
use database::models::U64;

use chrono::{Utc, TimeZone, Duration};

#[derive(Debug, Queryable, Identifiable)]
pub struct Timeout {
  pub id: i32,
  pub user_id: U64,
  pub server_id: U64,
  pub role_id: U64,
  pub seconds: i32,
  pub start: i64
}

impl Timeout {
  pub fn ends(&self) -> i64 {
    (Utc.timestamp(self.start as i64, 0) + Duration::seconds(self.seconds as i64)).timestamp()
  }
}

#[derive(Debug, Insertable)]
#[table_name = "timeouts"]
pub struct NewTimeout {
  pub user_id: String,
  pub server_id: String,
  pub role_id: String,
  pub seconds: i32,
  pub start: i64
}

impl NewTimeout {
  pub fn new(user_id: u64, server_id: u64, role_id: u64, seconds: i32, start: i64) -> Self {
    NewTimeout {
      user_id: user_id.to_string(),
      server_id: server_id.to_string(),
      role_id: role_id.to_string(),
      seconds: seconds,
      start: start
    }
  }
}
