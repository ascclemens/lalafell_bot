use database::schema::*;

use chrono::{Utc, TimeZone, Duration};

#[derive(Debug, Queryable, Identifiable)]
pub struct Timeout {
  pub id: i32,
  pub user_id: f64,
  pub server_id: f64,
  pub role_id: f64,
  pub seconds: i32,
  pub start: i32
}

impl Timeout {
  pub fn ends(&self) -> i64 {
    (Utc.timestamp(self.start as i64, 0) + Duration::seconds(self.seconds as i64)).timestamp()
  }
}
