use bot::BotEnv;
use filters::Filter;

use lalafell::commands::prelude::*;

use serenity::prelude::Mentionable;
use serenity::model::guild::Role;

use itertools::Itertools;

use chrono::Utc;

const USAGE: &str = "!search <filters>";

pub struct SearchCommand;

impl SearchCommand {
  pub fn new(_: Arc<BotEnv>) -> Self {
    SearchCommand
  }
}

#[derive(Debug, Deserialize)]
pub struct Params {
  filter_strings: Vec<String>
}

impl HasParams for SearchCommand {
  type Params = Params;
}

impl<'a> PublicChannelCommand<'a> for SearchCommand {
  fn run(&self, _: &Context, _: &Message, guild: GuildId, _: Arc<RwLock<GuildChannel>>, params: &[&str]) -> CommandResult<'a> {
    let params = self.params(USAGE, params)?;

    let filters = match Filter::all_filters(&params.filter_strings.join(" ")) {
      Some(f) => f,
      None => return Err("Invalid filters.".into())
    };
    let guild = some_or!(guild.find(), bail!("could not find guild"));
    let roles: Vec<Role> = guild.read().roles.values().cloned().collect();
    let now = Utc::now();
    let matches: Vec<String> = guild.read().members.values()
      .filter(|m| filters.iter().all(|f| f.matches(m, &roles)))
      .sorted_by(|a, b| a.display_name().cmp(&b.display_name()))
      .into_iter()
      .map(|m| format!("{} - {}",
        m.mention(),
        m.joined_at
          .map(|d| now.signed_duration_since(d))
          .map(|d| {
            let mut res = String::new();
            let seconds = d.num_seconds() % 60;
            let minutes = d.num_minutes() % 60;
            let hours = d.num_hours() % 24;
            let days = d.num_days();
            if days > 0 {
              res.push_str(&format!("{} day{}, ", days, if days == 1 { "" } else { "s" }));
            }
            if hours > 0 {
              res.push_str(&format!("{} hour{}, ", hours, if hours == 1 { "" } else { "s" }));
            }
            if minutes > 0 {
              res.push_str(&format!("{} minute{}, ", minutes, if minutes == 1 { "" } else { "s" }));
            }
            if seconds > 0 {
              res.push_str(&format!("{} second{}", seconds, if seconds == 1 { "" } else { "s" }));
            }
            res
          })
          .unwrap_or_else(|| String::from("unknown"))))
      .collect();
    let to_send = matches.join("\n");
    if to_send.len() > 2000 {
      return Err("Result was too large. Try a smaller filter.".into());
    }
    Ok(CommandSuccess::default()
      .message(move |e: CreateEmbed| e
        .description(to_send)
        .footer(|f| f.text(format!("{} member{}", matches.len(), if matches.len() == 1 { "" } else { "s" })))))
  }
}
