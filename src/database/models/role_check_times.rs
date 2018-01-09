use database::schema::*;
use database::models::U64;

insertable! {
  #[derive(Debug, Queryable, Identifiable, AsChangeset)]
  pub struct RoleCheckTime,
  #[derive(Debug, Insertable)]
  #[table_name = "role_check_times"]
  pub struct NewRoleCheckTime {
    pub check_id: i32,
    pub user_id: U64,
    pub reminded_at: i64,
    pub kick_after: i32
  }
}

impl NewRoleCheckTime {
  pub fn new<A, B, C, D>(check_id: A, user_id: B, reminded_at: C, kick_after: D) -> Self
    where A: Into<i32>,
          B: Into<U64>,
          C: Into<i64>,
          D: Into<i32>
  {
    NewRoleCheckTime {
      check_id: check_id.into(),
      user_id: user_id.into(),
      reminded_at: reminded_at.into(),
      kick_after: kick_after.into()
    }
  }
}
