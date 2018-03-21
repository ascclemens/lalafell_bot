use bot::BotEnv;

use lalafell::commands::prelude::*;
use lalafell::error::*;

use unicase::UniCase;

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
    let search_chars = res.characters.chain_err(|| "no characters field in search result")?.results;
    let uni_name = UniCase::new(name.as_str());
    let character = match search_chars.into_iter().find(|c| c["name"].as_str().map(UniCase::new) == Some(uni_name)) {
      Some(c) => c,
      None => return Err(format!("Could not find any character by the name {}.", name).into())
    };
    let character_id = character["id"].as_u64().chain_err(|| "invalid character object")?;
    let character = self.env.xivdb.character(character_id).chain_err(|| "could not download character")?;
    Ok(format!("{} ({})", character.data.race, character.data.clan).into())
  }
}
