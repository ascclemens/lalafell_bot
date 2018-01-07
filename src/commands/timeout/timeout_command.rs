use bot::BotEnv;
use commands::*;
use lalafell::error::*;
use database::models::NewTimeout;
use database::schema::timeouts;

use lalafell::commands::prelude::*;

use serenity::builder::CreateEmbed;
use serenity::model::channel::{Message, GuildChannel};
use serenity::model::misc::Mentionable;

use diesel::prelude::*;

use chrono::prelude::*;

use std::sync::Arc;

const USAGE: &str = "!timeout <who> <length>";

pub struct TimeoutCommand;

impl TimeoutCommand {
  pub fn new(_: Arc<BotEnv>) -> Self {
    TimeoutCommand
  }

  fn parse_duration(duration: &str) -> Result<u64> {
    let mut str_length = 0;
    let mut total_time = 0;
    while str_length < duration.len() {
      let numbers: String = duration.chars()
        .skip(str_length)
        .take_while(|c| c.is_numeric())
        .collect();
      str_length += numbers.len();
      let length: u64 = numbers.parse().chain_err(|| "could not parse duration length")?;
      let units: String = duration.chars()
        .skip(str_length)
        .take_while(|c| c.is_alphabetic() || c.is_whitespace())
        .collect();
      str_length += units.len();
      let multiplier = match units.trim().to_lowercase().as_ref() {
        "" if total_time == 0 => 1,
        "s" | "sec" | "secs" | "second" | "seconds" => 1,
        "m" | "min" | "mins" | "minute" | "minutes" => 60,
        "h" | "hr" | "hrs" | "hour" | "hours" => 3600,
        "d" | "ds" | "day" | "days" => 86400,
        _ => return Err("invalid unit".into())
      };
      total_time += length * multiplier;
    }
    Ok(total_time)
  }
}

#[derive(Debug, Deserialize)]
pub struct Params {
  who: MentionOrId,
  length: Vec<String>
}

impl HasParams for TimeoutCommand {
  type Params = Params;
}

impl<'a> PublicChannelCommand<'a> for TimeoutCommand {
  fn run(&self, _: &Context, message: &Message, guild: GuildId, channel: Arc<RwLock<GuildChannel>>, params: &[&str]) -> CommandResult<'a> {
    let params = self.params(USAGE, params)?;
    let member = guild.member(&message.author).chain_err(|| "could not get member")?;
    if !member.permissions().chain_err(|| "could not get permissions")?.manage_roles() {
      return Err(ExternalCommandFailure::default()
        .message(|e: CreateEmbed| e
          .title("Not enough permissions.")
          .description("You don't have enough permissions to use this command."))
        .wrap());
    }

    let server_id = channel.read().guild_id;
    let who = params.who;

    let mut timeout_member = match guild.member(*who) {
      Ok(m) => m,
      Err(_) => return Err("That user is not in this guild.".into())
    };

    let guild = some_or!(guild.find(), bail!("could not find guild"));

    let role_id = match timeout::set_up_timeouts(&guild.read()) {
      Ok(r) => {
        if let Err(e) = timeout_member.add_role(r) {
          warn!("could not add user {} to timeout role: {}", who.0, e);
        }
        r
      },
      Err(e) => {
        warn!("could not set up timeouts for {}: {}", guild.read().id.0, e);
        return Err("Could not set up timeouts for this server. Do I have enough permissions?".into());
      }
    };

    let duration = match TimeoutCommand::parse_duration(&params.length.into_iter().collect::<String>()) {
      Ok(d) => d,
      Err(_) => return Err("Invalid time length. Try \"15m\" or \"3 hours\" for example.".into())
    };

    let timeouts = ::bot::CONNECTION.with(|c| {
      use database::schema::timeouts::dsl;
      use diesel::expression::dsl::count;
      dsl::timeouts
        .filter(dsl::user_id.eq(who.0.to_string()).and(dsl::server_id.eq(server_id.0.to_string())))
        .select(count(dsl::id))
        .first(c)
        .optional()
        .chain_err(|| "could not load timeouts")
    })?;
    if timeouts.unwrap_or(0) > 0 {
      return Err(format!("{} is already timed out.", who.mention()).into());
    }

    let timeout_user = NewTimeout::new(who.0, server_id.0, role_id.0, duration as i32, Utc::now().timestamp());
    ::bot::CONNECTION.with(|c| ::diesel::insert_into(timeouts::table).values(&timeout_user).execute(c).chain_err(|| "could not insert timeout"))?;
    Ok(CommandSuccess::default())
  }
}
