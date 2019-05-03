use crate::database::{
  schema::*,
  models::Tag,
};

use uuid::Uuid;

insertable! {
  #[derive(Debug, Default, Queryable, Identifiable, Associations, AsChangeset)]
  #[belongs_to(Tag)]
  pub struct Verification,
  #[derive(Debug, Insertable)]
  #[table_name = "verifications"]
  pub struct NewVerification {
    pub tag_id: i32,
    pub verified: bool,
    pub verification_string: Option<String>,
  }
}

impl Verification {
  pub fn into_new(self, tag_id: i32) -> NewVerification {
    NewVerification {
      tag_id,
      verified: self.verified,
      verification_string: self.verification_string,
    }
  }
}

impl NewVerification {
  pub fn create_verification_string(&mut self) -> &str {
    let string = Uuid::new_v4().to_simple().to_string();
    self.verification_string = Some(string);
    self.verification_string.as_ref().unwrap()
  }
}
