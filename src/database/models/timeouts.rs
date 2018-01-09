use database::schema::*;
use database::models::U64;

use chrono::{Utc, TimeZone, Duration};

insertable! {
  #[derive(Debug, Queryable, Identifiable)]
  pub struct Timeout,
  #[derive(Debug, Insertable)]
  #[table_name = "timeouts"]
  pub struct NewTimeout {
    pub user_id: U64,
    pub server_id: U64,
    pub role_id: U64,
    pub seconds: i32,
    pub start: i64
  }
}

impl Timeout {
  pub fn ends(&self) -> i64 {
    (Utc.timestamp(self.start, 0) + Duration::seconds(i64::from(self.seconds))).timestamp()
  }
}

impl NewTimeout {
  pub fn new(user_id: u64, server_id: u64, role_id: u64, seconds: i32, start: i64) -> Self {
    NewTimeout {
      user_id: user_id.into(),
      server_id: server_id.into(),
      role_id: role_id.into(),
      seconds,
      start
    }
  }
}
