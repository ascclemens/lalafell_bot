use crate::bot::BotEnv;

use failure::Fail;

use ffxiv::World;

use lalafell::commands::prelude::*;
use lalafell::error::*;

use unicase::UniCase;

use lodestone_api_client::{
  prelude::*,
  models::RouteResult,
};

#[derive(BotCommand)]
pub struct RaceCommand {
  env: Arc<BotEnv>,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Display the race of the specified character")]
pub struct Params {
  #[structopt(help = "The world the character is on")]
  world: World,
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
    let params = self.params_then("race", params, |a| a.setting(structopt::clap::AppSettings::ArgRequiredElseHelp))?;
    let world = params.world;
    let name = format!("{} {}", params.first_name, params.last_name);

    let res = self.env.lodestone
      .character_search()
      .name(&name)
      .world(world)
      .send()
      .map_err(Fail::compat)
      .chain_err(|| "could not search Lodestone API")?;
    let res = match res {
      RouteResult::Success { result, .. } | RouteResult::Scraped { result } | RouteResult::Cached { result, .. } => result,
      RouteResult::Error { error } => return Err(format!("An error occurred: `{}`. Try again later.", error).into()),
      _ => bail!("bad routeresult: {:#?}", res),
    };
    let uni_name = UniCase::new(name.as_str());
    let character = match res.results.into_iter().find(|c| UniCase::new(&c.name) == uni_name) {
      Some(c) => c,
      None => return Err(format!("Could not find any character by the name {}.", name).into())
    };
    let res = self.env.lodestone
      .character(character.id.into())
      .send()
      .map_err(Fail::compat)
      .chain_err(|| "could not download character")?;

    let character = match res {
      RouteResult::Cached { result, .. } | RouteResult::Scraped { result } | RouteResult::Success { result, .. } => result,
      RouteResult::NotFound => return Err("No such character.".into()),
      RouteResult::Adding { .. } => return Err("That character is not in the database. Try again in one minute.".into()),
      RouteResult::Error { error } => return Err(format!("An error occurred: `{}`. Try again later.", error).into()),
    };

    Ok(format!("{} ({})", character.race.name(), character.clan.name()).into())
  }
}
