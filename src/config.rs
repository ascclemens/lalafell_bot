use serde_json::Value;

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
  pub tasks: Value
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Presence {
  pub change_frequency: i64,
  pub list: Vec<PresenceEntry>
}

#[derive(Debug, Clone, Deserialize)]
pub struct PresenceEntry {
  #[serde(rename = "type")]
  pub kind: PresenceKind,
  pub content: String,
  #[serde(default)]
  pub url: Option<String>
}

#[derive(Debug, Clone, Deserialize)]
pub enum PresenceKind {
  #[serde(rename = "playing")]
  Playing,
  #[serde(rename = "streaming")]
  Streaming,
  #[serde(rename = "listening")]
  Listening
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
