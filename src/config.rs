use serde_json::Value;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Config {
  pub bot: Bot,
  pub roles: Roles,
  pub tasks: Vec<Task>,
  pub listeners: Vec<Listener>
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Bot {
  pub administrators: Vec<u64>,
  pub timeouts: Timeouts
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Timeouts {
  pub timed_out_role: Option<String>,
  pub role_check_interval: Option<i64>
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Roles {
  pub groups: Vec<Vec<String>>
}

#[derive(Debug, Deserialize)]
pub struct Task {
  pub name: String,
  #[serde(default)]
  pub config: Option<Value>
}

#[derive(Debug, Deserialize)]
pub struct Listener {
  pub name: String,
  pub config: Option<Value>
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct DeleteAllMessagesTask {
  pub except: Vec<u64>
}
