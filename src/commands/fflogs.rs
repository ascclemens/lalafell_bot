use lalafell::commands::prelude::*;

use lalafell::error::*;

use fflogs::{self, FfLogs};
use fflogs::net::{ServerRegion, Metric};

use std::cmp::Ordering;

pub struct FfLogsCommand {
  fflogs: FfLogs
}

impl ::commands::BotCommand for FfLogsCommand {
  fn new(env: Arc<::bot::BotEnv>) -> Self {
    FfLogsCommand {
      fflogs: FfLogs::new(&env.environment.fflogs_api_key)
    }
  }
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Display information about the character on FF Logs")]
pub struct Params {
  #[structopt(help = "The server the character is on")]
  server: String,
  #[structopt(help = "The character's first name")]
  first_name: String,
  #[structopt(help = "The character's last name")]
  last_name: String
}

impl HasParams for FfLogsCommand {
  type Params = Params;
}

impl<'a> Command<'a> for FfLogsCommand {
  fn run(&self, _: &Context, _: &Message, params: &[&str]) -> CommandResult<'a> {
    let params = self.params_then("fflogs", params, |a| a.setting(::structopt::clap::AppSettings::ArgRequiredElseHelp))?;
    let server = params.server;
    let region = match FfLogsCommand::region(&server) {
      Some(r) => r,
      None => return Err("Invalid server.".into())
    };
    let name = format!("{} {}", params.first_name, params.last_name);

    let parses = self.fflogs.parses(
      &name,
      &server,
      region,
      |x| x.metric(Metric::Dps)
    );

    let parses = match parses {
      Ok(r) => r,
      Err(fflogs::errors::Error::FfLogs(fflogs::net::FfLogsError { status, error })) => {
        return Err(format!("FF Logs didn't like that ({}): {}", status, error).into());
      },
      Err(e) => return Err(e).chain_err(|| "could not query fflogs")?
    };

    if parses.is_empty() {
      return Ok("No parses found.".into());
    }

    let first_spec = match parses[0].specs.get(0) {
      Some(s) => s,
      None => return Err("Somehow there was no first spec.".into())
    };
    let first_data = match first_spec.data.get(0) {
      Some(d) => d,
      None => return Err("Somehow there was no first data.".into())
    };

    let job = &first_spec.spec;
    let name = &first_data.character_name;
    let id = first_data.character_id;

    let mut embed = CreateEmbed::default();

    embed = embed
      .title(&name)
      .url(format!("https://www.fflogs.com/character/id/{}", id))
      .field("Job", &job, true)
      .field("Server", &server, true);

    for parse in &parses {
      let spec = match parse.specs.iter().filter(|s| s.spec == *job).next() {
        Some(s) => s,
        None => continue
      };
      let data = match spec.data.iter().max_by(|a, b| a.historical_percent.partial_cmp(&b.historical_percent).unwrap_or(Ordering::Less)) {
        Some(d) => d,
        None => continue
      };

      let url = format!("https://www.fflogs.com/reports/{}#fight={}", data.report_code, data.report_fight);

      let string = format!("[Link]({url}) – {dps:.2} DPS – {perc:.2} percentile out of {total_parses} parses",
        url = url,
        dps = data.persecondamount,
        perc = data.historical_percent,
        total_parses = data.historical_count
      );

      embed = embed.field(&parse.name, string, false);
    }
    Ok(CommandSuccess::default().message(|_| embed))
  }
}

impl FfLogsCommand {
  fn region(server: &str) -> Option<ServerRegion> {
    match server.to_lowercase().as_str() {
      "adamantoise" | "balmung" | "cactuar" | "coeurl" | "faerie" | "gilgamesh" | "goblin" | "jenova" | "mateus" | "midgardsormr" | "sargatanas" | "siren" | "zalera"
      | "behemoth" | "brynhildr" | "diabolos" | "excalibur" | "exodus" | "famfrit" | "hyperion" | "lamia" | "leviathan" | "malboro" | "ultros"
        => Some(ServerRegion::NorthAmerica),
      "aegis" | "atomos" | "carbuncle" | "garuda" | "gungnir" | "kujata" | "ramuh" | "tonberry" | "typhon" | "unicorn"
      | "alexander" | "bahamut" | "durandal" | "fenrir" | "ifrit" | "ridill" | "tiamat" | "ultima" | "valefor" | "yojimbo" | "zeromus"
      | "anima" | "asura" | "belias" | "chocobo" | "hades" | "ixion" | "mandragora" | "masamune" | "pandaemonium" | "shinryu" | "titan"
        => Some(ServerRegion::Japan),
      "cerberus" | "lich" | "louisoix" | "moogle" | "odin" | "omega" | "phoenix" | "ragnarok" | "shiva" | "zodiark"
        => Some(ServerRegion::Europe),
      _ => None
    }
  }
}
