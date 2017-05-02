#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Config {
  pub roles: Roles,
  pub delete_all_messages_task: DeleteAllMessagesTask
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Roles {
  pub groups: Vec<Vec<String>>
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct DeleteAllMessagesTask {
  pub except: Vec<u64>
}
