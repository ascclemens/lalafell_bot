use crate::filters::Filter;

use lalafell::{
  commands::prelude::*,
  error::*,
};

use serenity::{
  model::guild::Role,
  prelude::Mentionable,
};

use itertools::Itertools;

use chrono::Utc;

#[derive(BotCommand)]
pub struct SearchCommand;

#[derive(Debug, StructOpt)]
#[structopt(about = "Search the members of the server with filters")]
pub struct Params {
  #[structopt(name = "filters", help = "A list of filters to apply when searching")]
  filter_strings: Vec<String>,
}

impl HasParams for SearchCommand {
  type Params = Params;
}

impl<'a> PublicChannelCommand<'a> for SearchCommand {
  fn run(&self, ctx: &Context, _: &Message, guild: GuildId, _: Arc<RwLock<GuildChannel>>, params: &[&str]) -> CommandResult<'a> {
    let params = self.params_then("search", params, |a| a.setting(structopt::clap::AppSettings::ArgRequiredElseHelp))?;

    let filters = match Filter::all_filters(&params.filter_strings.join(" ")) {
      Some(f) => f,
      None => return Err("Invalid filters.".into()),
    };
    let guild = guild.to_guild_cached(&ctx).chain_err(|| "could not find guild")?;
    let reader = guild.read();
    let roles: Vec<&Role> = reader.roles.values().collect();
    let now = Utc::now();
    let matches: Vec<String> = guild.read().members.values()
      .filter(|m| filters.iter().all(|f| f.matches(m, &roles)))
      .sorted_by(|a, b| a.display_name().cmp(&b.display_name()))
      .map(|m| format!(
        "{} - {}",
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
          .unwrap_or_else(|| String::from("unknown")),
      ))
      .collect();
    let to_send = matches.join("\n");
    if to_send.len() > 2000 {
      return Err("Result was too large. Try a smaller filter.".into());
    }
    Ok(CommandSuccess::default()
      .message(move |e: &mut CreateEmbed| e
        .description(to_send)
        .footer(|f| f.text(format!("{} member{}", matches.len(), if matches.len() == 1 { "" } else { "s" })))))
  }
}
