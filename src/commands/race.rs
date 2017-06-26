use bot::LalafellBot;
use commands::*;

use error::*;

use std::sync::Arc;

const USAGE: &'static str = "!race <server> <character>";

pub struct RaceCommand {
  bot: Arc<LalafellBot>
}

impl RaceCommand {
  pub fn new(bot: Arc<LalafellBot>) -> RaceCommand {
    RaceCommand {
      bot: bot
    }
  }
}

#[derive(Debug, Deserialize)]
pub struct Params {
  server: String,
  name: (String, String)
}

impl HasParams for RaceCommand {
  type Params = Params;
}

impl<'a> Command<'a> for RaceCommand {
  fn run(&self, _: &Message, params: &[&str]) -> CommandResult<'a> {
    let params = self.params(USAGE, params)?;
    let server = params.server;
    let name = params.name.0 + " " + &params.name.1;
    let params = &[
      ("one", "characters"),
      ("strict", "on"),
      ("server|et", &server)
    ];
    let res = self.bot.xivdb.search(&name, params).chain_err(|| "could not search XIVDB")?;
    let search_chars = match res.characters {
      Some(c) => c.results,
      None => {
        let err: error::Error = "no characters field in search result".into();
        return Err(err.into());
      }
    };
    if search_chars.is_empty() {
      return Err(format!("Could not find any character by the name {}.", name).into());
    }
    let character = self.bot.xivdb.character(search_chars[0]["id"].as_u64().unwrap()).unwrap();
    if character.name.to_lowercase() != name.to_lowercase() {
      return Err(format!("Could not find any character by the name {}.", name).into());
    }
    Ok(format!("{} ({})", character.data.race, character.data.clan).into())
  }
}
