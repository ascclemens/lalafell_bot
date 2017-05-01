#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Config {
  pub roles: Roles
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Roles {
  pub groups: Vec<Vec<String>>
}
