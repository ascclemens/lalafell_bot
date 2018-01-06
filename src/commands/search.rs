use bot::BotEnv;
use filters::Filter;

use lalafell::commands::prelude::*;
use lalafell::error::*;

use serenity::prelude::Mentionable;
use serenity::model::guild::Role;
use serenity::builder::CreateEmbed;

use chrono::Utc;

const USAGE: &'static str = "!search <filters>";

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
  fn run(&self, _: &Context, message: &Message, guild: GuildId, _: Arc<RwLock<GuildChannel>>, params: &[&str]) -> CommandResult<'a> {
    let params = self.params(USAGE, params)?;

    let member = guild.member(&message.author).chain_err(|| "could not get member")?;
    if !member.permissions().chain_err(|| "could not get permissions")?.manage_roles() {
      return Err(ExternalCommandFailure::default()
        .message(|e: CreateEmbed| e
          .title("Not enough permissions.")
          .description("You don't have enough permissions to use this command."))
        .wrap());
    }

    let filters: Vec<Filter> = match params.filter_strings.iter().map(|x| Filter::parse(x)).collect::<Option<_>>() {
      Some(f) => f,
      None => return Err("Invalid filter.".into())
    };
    let guild = some_or!(guild.find(), bail!("could not find guild"));
    let roles: Vec<Role> = guild.read().roles.values().cloned().collect();
    let now = Utc::now();
    let matches: Vec<String> = guild.read().members.values()
      .filter(|m| filters.iter().all(|f| f.matches(m, &roles)))
      .map(|m| format!("{} - {}",
        m.mention(),
        m.joined_at
          .map(|d| now.clone().signed_duration_since(d))
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
    Ok(matches.join("\n").into())
  }
}
