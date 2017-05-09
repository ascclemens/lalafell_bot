use uuid::Uuid;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Database {
  #[serde(skip_serializing, skip_deserializing)]
  pub last_saved: i64,
  pub autotags: Autotags
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Autotags {
  pub last_updated: i64,
  pub users: Vec<AutotagUser>
}

impl Autotags {
  pub fn update_or_add(&mut self, user: AutotagUser) {
    for u in &mut self.users {
      if u.user_id == user.user_id && u.server_id == user.server_id {
        u.character_id = user.character_id;
        u.character = user.character;
        u.server = user.server;
        return;
      }
    }
    self.users.push(user);
  }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AutotagUser {
  pub user_id: u64,
  pub server_id: u64,
  pub character_id: u64,
  pub character: String,
  pub server: String,
  #[serde(default)]
  pub verification: UserVerification
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UserVerification {
  pub verified: bool,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub verification_string: Option<String>
}

impl UserVerification {
  pub fn create_verification_string(&mut self) -> &String {
    let uuid = Uuid::new_v4().simple().to_string();
    self.verification_string = Some(uuid);
    self.verification_string.as_ref().unwrap()
  }
}

impl AutotagUser {
  pub fn new(user_id: u64, server_id: u64, character_id: u64, character: &str, server: &str) -> AutotagUser {
    AutotagUser {
      user_id: user_id,
      server_id: server_id,
      character_id: character_id,
      character: character.to_string(),
      server: server.to_string(),
      verification: Default::default()
    }
  }
}
