use serde_json::Value;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Config {
  pub timeouts: Timeouts,
  pub presence: Presence,
  pub tasks: Value
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
