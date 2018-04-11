use ffxiv::{DataCenter, World, Role, Job};

use lalafell::commands::prelude::*;
use lalafell::error::*;

use serenity::builder::CreateEmbed;

use unicase::UniCase;

use std::str::FromStr;
use std::sync::Arc;

#[derive(BotCommand)]
pub struct PartyFinderCommand;

#[derive(Debug, StructOpt)]
#[structopt(about = "Create a party finder advertisement")]
pub struct Params {
  #[structopt(
    short = "m",
    long = "message",
    help = "The message of the party finder",
  )]
  message: Option<String>,
  #[structopt(
    short = "s",
    long = "size",
    help = "The size of the party finder",
    raw(possible_values = r#"&["2", "3", "4", "5", "6", "7", "8"]"#),
    raw(hide_possible_values = "true"),
  )]
  size: Option<u8>,
  #[structopt(
    short = "d",
    long = "data-center",
    help = "The data center the party finder is for",
  )]
  data_center: Option<DataCenter>,
  #[structopt(
    short = "w",
    long = "world",
    alias = "server",
    help = "The world the party finder is for (cannot be used with --data-center)",
  )]
  world: Option<World>,
  #[structopt(
    short = "i",
    long = "duty",
    help = "The duty the party finder is for",
  )]
  duty: Option<PartyFinderDuty>,
  #[structopt(
    short = "l",
    long = "min-ilvl",
    help = "The minimum item level for the party",
  )]
  min_ilvl: Option<u16>,
  #[structopt(
    short = "j",
    long = "job",
    help = "A job required in this party (can be specified multiple times)",
  )]
  jobs: Vec<Slot>,
}

impl HasParams for PartyFinderCommand {
  type Params = Params;
}

impl<'a> PublicChannelCommand<'a> for PartyFinderCommand {
  fn run(&self, _: &Context, msg: &Message, gid: GuildId, _: Arc<RwLock<GuildChannel>>, params: &[&str]) -> CommandResult<'a> {
    let params = self.params_then("partyfinder", params, |a| a
      .group(::structopt::clap::ArgGroup::with_name("location")
        .args(&["data_center", "world"])
        .required(true))
      .group(::structopt::clap::ArgGroup::with_name("size_jobs")
        .args(&["size", "jobs"])
        .required(true))
      .setting(::structopt::clap::AppSettings::ArgRequiredElseHelp))?;

    let member = gid.member(&msg.author).chain_err(|| "could not get member")?;

    let mut embed = CreateEmbed::default()
      .author(|a| a
        .name(&member.display_name())
        .icon_url(&member.user.read().face()));
    if let Some(message) = params.message {
      embed = embed.description(message);
    }
    if let Some(duty) = params.duty {
      embed = embed.field("Duty", duty.to_string(), false);
    }
    if !params.jobs.is_empty() {
      let jobs = params.jobs
        .into_iter()
        .map(|j| match j {
          Slot::All => "All".into(),
          Slot::Job(js) => js.into_iter().map(|x| ICONS[x.as_code()]).collect::<Vec<_>>().join(""),
          Slot::Role(rs) => rs.into_iter().map(|x| x.as_str()).collect::<Vec<_>>().join(""),
        })
        .collect::<Vec<_>>()
        .join("|");
      embed = embed.field("Jobs", jobs, false);
    }
    if let Some(min_ilvl) = params.min_ilvl {
      embed = embed.field("Minimum item level", min_ilvl, true);
    }
    if let Some(data_center) = params.data_center {
      embed = embed.field("Data center", data_center, true);
    }
    if let Some(world) = params.world {
      embed = embed.field("World", world, true);
    }

    if let Err(e) = msg.delete() {
      warn!("could not delete party finder command message ({}): {}", msg.id, e);
    }

    Ok(CommandSuccess::default().message(|_| embed))
  }
}

#[derive(Debug)]
enum PartyFinderDuty {
  DeltascapeV10(Difficulty),
  DeltascapeV20(Difficulty),
  DeltascapeV30(Difficulty),
  DeltascapeV40(Difficulty),

  SigmascapeV10(Difficulty),
  SigmascapeV20(Difficulty),
  SigmascapeV30(Difficulty),
  SigmascapeV40(Difficulty),
}

impl ToString for PartyFinderDuty {
  fn to_string(&self) -> String {
    if let Some(diff) = self.difficulty() {
      if *diff != Difficulty::Normal {
        return format!("{} ({})", self.base_str(), diff.as_str());
      }
    }
    self.base_str().to_string()
  }
}

impl PartyFinderDuty {
  fn base_str(&self) -> &'static str {
    match *self {
      PartyFinderDuty::DeltascapeV10(_) => "Deltascape V1.0",
      PartyFinderDuty::DeltascapeV20(_) => "Deltascape V2.0",
      PartyFinderDuty::DeltascapeV30(_) => "Deltascape V3.0",
      PartyFinderDuty::DeltascapeV40(_) => "Deltascape V4.0",

      PartyFinderDuty::SigmascapeV10(_) => "Sigmascape V1.0",
      PartyFinderDuty::SigmascapeV20(_) => "Sigmascape V2.0",
      PartyFinderDuty::SigmascapeV30(_) => "Sigmascape V3.0",
      PartyFinderDuty::SigmascapeV40(_) => "Sigmascape V4.0",
    }
  }

  fn difficulty(&self) -> Option<&Difficulty> {
    let difficulty = match *self {
      PartyFinderDuty::DeltascapeV10(ref d) => d,
      PartyFinderDuty::DeltascapeV20(ref d) => d,
      PartyFinderDuty::DeltascapeV30(ref d) => d,
      PartyFinderDuty::DeltascapeV40(ref d) => d,

      PartyFinderDuty::SigmascapeV10(ref d) => d,
      PartyFinderDuty::SigmascapeV20(ref d) => d,
      PartyFinderDuty::SigmascapeV30(ref d) => d,
      PartyFinderDuty::SigmascapeV40(ref d) => d,
    };
    Some(difficulty)
  }
}

impl FromStr for PartyFinderDuty {
  type Err = String;

  fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
    let duty = match s.to_lowercase().as_str() {
      "o1" => PartyFinderDuty::DeltascapeV10(Difficulty::Normal),
      "o2" => PartyFinderDuty::DeltascapeV20(Difficulty::Normal),
      "o3" => PartyFinderDuty::DeltascapeV30(Difficulty::Normal),
      "o4" => PartyFinderDuty::DeltascapeV40(Difficulty::Normal),

      "o5" => PartyFinderDuty::SigmascapeV10(Difficulty::Normal),
      "o6" => PartyFinderDuty::SigmascapeV20(Difficulty::Normal),
      "o7" => PartyFinderDuty::SigmascapeV30(Difficulty::Normal),
      "o8" => PartyFinderDuty::SigmascapeV40(Difficulty::Normal),

      "o1s" => PartyFinderDuty::DeltascapeV10(Difficulty::Savage),
      "o2s" => PartyFinderDuty::DeltascapeV20(Difficulty::Savage),
      "o3s" => PartyFinderDuty::DeltascapeV30(Difficulty::Savage),
      "o4s" => PartyFinderDuty::DeltascapeV40(Difficulty::Savage),

      "o5s" => PartyFinderDuty::SigmascapeV10(Difficulty::Savage),
      "o6s" => PartyFinderDuty::SigmascapeV20(Difficulty::Savage),
      "o7s" => PartyFinderDuty::SigmascapeV30(Difficulty::Savage),
      "o8s" => PartyFinderDuty::SigmascapeV40(Difficulty::Savage),

      _ => return Err(format!("Unknown duty {}", s))
    };

    Ok(duty)
  }
}

#[derive(Debug, PartialEq)]
enum Difficulty {
  Normal,
  Hard,
  Extreme,
  Savage,
  Ultimate,
}

impl Difficulty {
  fn as_str(&self) -> &'static str {
    match *self {
      Difficulty::Normal => "Normal",
      Difficulty::Hard => "Hard",
      Difficulty::Extreme => "Extreme",
      Difficulty::Savage => "Savage",
      Difficulty::Ultimate => "Ultimate",
    }
  }
}

#[derive(Debug)]
enum Slot {
  All,
  Role(Vec<Role>),
  Job(Vec<Job>)
}

impl Slot {
  fn parse_all_roles<I, S>(s: I) -> ::std::result::Result<Vec<Role>, ::ffxiv::errors::UnknownVariant>
    where I: IntoIterator<Item=S>,
          S: AsRef<str>,
  {
    let mut parts = Vec::new();
    for part in s.into_iter() {
      parts.push(Role::from_str(part.as_ref().trim())?);
    }
    Ok(parts)
  }

  fn parse_all_jobs<I, S>(s: I) -> ::std::result::Result<Vec<Job>, ::ffxiv::errors::UnknownVariant>
    where I: IntoIterator<Item=S>,
          S: AsRef<str>,
  {
    let mut parts = Vec::new();
    for part in s.into_iter() {
      parts.push(Job::from_str(part.as_ref().trim())?);
    }
    Ok(parts)
  }
}

impl FromStr for Slot {
  type Err = String;

  fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
    if UniCase::new(s) == UniCase::new("all") {
      return Ok(Slot::All);
    }

    let parts: Vec<&str> = s.split(|c| c == '+' || c == '|' || c == '/').collect();

    Slot::parse_all_roles(&parts)
      .map(Slot::Role)
      .or_else(|_| Slot::parse_all_jobs(&parts)
        .map(Slot::Job))
      .map_err(|_| "Slot was not all roles/jobs".to_string())
  }
}

lazy_static! {
  static ref ICONS: ::std::collections::HashMap<&'static str, &'static str> = {
    let mut map = ::std::collections::HashMap::new();
    map.insert("DRK", "<:drk:430347698856525826>");
    map.insert("AST", "<:ast:430347698978291712>");
    map.insert("WHM", "<:whm:430347699028754434>");
    map.insert("DRG", "<:drg:430347699032686593>");
    map.insert("MCH", "<:mch:430347699087212545>");
    map.insert("BLM", "<:blm:430347699133480970>");
    map.insert("WAR", "<:war:430347699175292929>");
    map.insert("MNK", "<:mnk:430347699213041674>");
    map.insert("BRD", "<:brd:430347699221561344>");
    map.insert("PLD", "<:pld:430347699242663948>");
    map.insert("NIN", "<:nin:430347699263373343>");
    map.insert("SAM", "<:sam:430347699326418944>");
    map.insert("SCH", "<:sch:430347699334676490>");
    map.insert("RDM", "<:rdm:430347699389464576>");
    map.insert("SMN", "<:smn:430347699443990549>");
    map
  };
}
