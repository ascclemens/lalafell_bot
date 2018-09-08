use bot::BotEnv;
use commands::tag::CharacterResult;

use failure::Fail;

use ffxiv::World;

use lalafell::commands::prelude::*;
use lalafell::error::*;

use unicase::UniCase;

use xivapi::{
  prelude::*,
  models::{
    State,
    character::{Race, Tribe},
  },
};

#[derive(BotCommand)]
pub struct RaceCommand {
  env: Arc<BotEnv>
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Display the race of the specified character")]
pub struct Params {
  #[structopt(help = "The server the character is on")]
  server: World,
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

    let res = self.env.xivapi
      .character_search()
      .name(&name)
      .server(server)
      .tags(&["race"])
      .send()
      .map_err(|x| x.compat())
      .chain_err(|| "could not search XIVAPI")?;
    let uni_name = UniCase::new(name.as_str());
    let character = match res.results.into_iter().find(|c| UniCase::new(&c.name) == uni_name) {
      Some(c) => c,
      None => return Err(format!("Could not find any character by the name {}.", name).into())
    };
    let res: CharacterResult = self.env.xivapi
      .character(character.id.into())
      .columns(&["Info.Character", "Character.ID", "Character.Name", "Character.Race", "Character.Gender", "Character.Server", "Character.Tribe"])
      .tags(&["race"])
      .json()
      .map_err(|x| x.compat())
      .chain_err(|| "could not download character")?;

    match res.info.character.state {
      State::Adding => return Err("That character is not in the database. Try again in two minutes.".into()),
      State::NotFound => return Err("No such character.".into()),
      State::Blacklist => return Err("That character has removed themselves from the database.".into()),
      _ => {},
    }

    let character = match res.character {
      Some(c) => c,
      None => bail!("missing character"),
    };

    let race = match character.race {
      Race::Hyur => "Hyur",
      Race::Elezen => "Elezen",
      Race::Lalafell => "Lalafell",
      Race::Miqote => "Miqo'te",
      Race::Roegadyn => "Roegadyn",
      Race::AuRa => "Au Ra",
    };

    let tribe = match character.tribe {
      Tribe::Midlander => "Midlander",
      Tribe::Highlander => "Highlander",
      Tribe::Wildwood => "Wildwood",
      Tribe::Duskwight => "Duskwight",
      Tribe::Plainsfolk => "Plainsfolk",
      Tribe::Dunesfolk => "Dunesfolk",
      Tribe::SeekerOfTheSun => "Seeker of the Sun",
      Tribe::SeekerOfTheMoon => "Seeker of the Moon",
      Tribe::SeaWolf => "Sea Wolf",
      Tribe::Hellsguard => "Hellsguard",
      Tribe::Raen => "Raen",
      Tribe::Xaela => "Xaela",
    };

    Ok(format!("{} ({})", race, tribe).into())
  }
}
