use bot::BotEnv;

use lalafell::commands::prelude::*;

use lalafell::error;
use lalafell::error::*;

const USAGE: &str = "!race <server> <character>";

pub struct RaceCommand {
  env: Arc<BotEnv>
}

impl RaceCommand {
  pub fn new(env: Arc<BotEnv>) -> RaceCommand {
    RaceCommand { env }
  }
}

#[derive(Debug, Deserialize)]
pub struct Params {
  server: String,
  name: [String; 2]
}

impl HasParams for RaceCommand {
  type Params = Params;
}

impl<'a> Command<'a> for RaceCommand {
  fn run(&self, _: &Context, _: &Message, params: &[&str]) -> CommandResult<'a> {
    let params = self.params(USAGE, params)?;
    let server = params.server;
    let name = params.name.join(" ");
    let params = &[
      ("one", "characters"),
      ("strict", "on"),
      ("server|et", &server)
    ];
    let res = self.env.xivdb.search(&name, params).chain_err(|| "could not search XIVDB")?;
    let search_chars = match res.characters {
      Some(c) => c.results,
      None => return Err(into!(error::Error, "no characters field in search result").into())
    };
    if search_chars.is_empty() {
      return Err(format!("Could not find any character by the name {}.", name).into());
    }
    let character = self.env.xivdb.character(search_chars[0]["id"].as_u64().unwrap()).unwrap();
    if character.name.to_lowercase() != name.to_lowercase() {
      return Err(format!("Could not find any character by the name {}.", name).into());
    }
    Ok(format!("{} ({})", character.data.race, character.data.clan).into())
  }
}
