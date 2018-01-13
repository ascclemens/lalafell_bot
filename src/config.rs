use serde_json::Value;

use std::collections::HashMap;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Config {
  pub bot: Bot,
  pub roles: Roles
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Bot {
  pub administrators: Vec<u64>,
  pub timeouts: Timeouts,
  pub presence: Presence,
  pub tasks: Value,
  pub log: HashMap<u64, u64>
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Presence {
  pub change_frequency: i64
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Timeouts {
  pub role_check_interval: Option<i64>
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Roles {
  pub groups: Vec<Vec<String>>
}
