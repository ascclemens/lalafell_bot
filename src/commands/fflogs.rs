use lalafell::commands::prelude::*;

use lalafell::error::*;

use fflogs::{self, FfLogs};
use fflogs::models::classes::Classes;
use fflogs::models::zones::Zones;
use fflogs::net::{ServerRegion, Metric};

use serenity::prelude::RwLock;

pub struct FfLogsCommand {
  fflogs: FfLogs,
  zones: RwLock<Option<Zones>>,
  classes: RwLock<Option<Classes>>
}

impl ::commands::BotCommand for FfLogsCommand {
  fn new(env: Arc<::bot::BotEnv>) -> Self {
    FfLogsCommand {
      fflogs: FfLogs::new(&env.environment.fflogs_api_key),
      zones: Default::default(),
      classes: Default::default()
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
    self.check_zones_classes()?;

    let params = self.params_then("fflogs", params, |a| a.setting(::structopt::clap::AppSettings::ArgRequiredElseHelp))?;
    let server = params.server;
    let region = match FfLogsCommand::region(&server) {
      Some(r) => r,
      None => return Err("Invalid server.".into())
    };
    let name = format!("{} {}", params.first_name, params.last_name);

    let rankings = self.fflogs.rankings_character(
      &name,
      &server,
      region,
      |x| x.metric(Metric::Dps)
    );

    let rankings = match rankings {
      Ok(r) => r,
      Err(fflogs::errors::Error::FfLogs(fflogs::net::FfLogsError { status, error })) => {
        return Err(format!("FF Logs didn't like that ({}): {}", status, error).into());
      },
      Err(e) => return Err(e).chain_err(|| "could not query fflogs")?
    };

    if rankings.is_empty() {
      return Ok("No parses found.".into());
    }

    let zones = self.zones.read();
    let zones = zones.as_ref().chain_err(|| "no zones")?;
    let classes = self.classes.read();
    let classes = classes.as_ref().chain_err(|| "no classes")?;

    let class = match classes.iter().find(|c| c.id == rankings[0].class) {
      Some(c) => c,
      None => return Err("Could not find class.".into())
    };
    let spec = match class.specs.iter().find(|s| s.id == rankings[0].spec) {
      Some(s) => s,
      None => return Err("Could not find class specification.".into())
    };

    let mut embed = CreateEmbed::default();

    embed = embed
      .title(&name)
      .field("Job", &spec.name, true)
      .field("Server", &server, true);

    for ranking in rankings.into_iter().filter(|r| r.class == class.id && r.spec == spec.id) {
      let encounter = match zones.iter().flat_map(|z| z.encounters.iter()).find(|e| e.id == ranking.encounter) {
        Some(n) => n,
        None => continue
      };

      let url = format!("https://www.fflogs.com/reports/{}#fight={}", ranking.report_id, ranking.fight_id);

      let string = format!("[Link]({}) – {:.2} DPS – {}/{} parses (T%: {:.2})",
        url,
        ranking.total,
        ranking.rank,
        ranking.out_of,
        (1.0 - ranking.rank as f32 / ranking.out_of as f32) * 100.0
      );

      embed = embed.field(&encounter.name, string, false);
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

  fn check_zones_classes(&self) -> Result<()> {
    if self.zones.read().is_none() {
      *self.zones.write() = Some(self.fflogs.zones().chain_err(|| "could not download zones")?);
    }
    if self.classes.read().is_none() {
      *self.classes.write() = Some(self.fflogs.classes().chain_err(|| "could not download classes")?);
    }
    Ok(())
  }
}
