use bot::BotEnv;

use lalafell::commands::prelude::*;

use lalafell::error;
use lalafell::error::*;

#[derive(BotCommand)]
pub struct RaceCommand {
  env: Arc<BotEnv>
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Display the race of the specified character")]
pub struct Params {
  #[structopt(help = "The server the character is on")]
  server: String,
  #[structopt(help = "The character's first name")]
  first_name: String,
  #[structopt(help = "The character's last name")]
  last_name: String
}

impl HasParams for RaceCommand {
  type Params = Params;
}

impl<'a> Command<'a> for RaceCommand {
  fn run(&self, _: &Context, _: &Message, params: &[&str]) -> CommandResult<'a> {
    let params = self.params_then("race", params, |a| a.setting(::structopt::clap::AppSettings::ArgRequiredElseHelp))?;
    let server = params.server;
    let name = format!("{} {}", params.first_name, params.last_name);
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
