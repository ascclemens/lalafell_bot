use bot::LalafellBot;
use commands::*;
use error::*;
use database::TimeoutUser;

use discord::builders::EmbedBuilder;
use discord::model::{PublicChannel, UserId};
use discord::model::permissions;

use chrono::prelude::*;

use std::sync::Arc;

const USAGE: &'static str = "!timeout <who> <length>";

pub struct TimeoutCommand {
  bot: Arc<LalafellBot>
}

impl TimeoutCommand {
  pub fn new(bot: Arc<LalafellBot>) -> TimeoutCommand {
    TimeoutCommand {
      bot: bot
    }
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

impl HasBot for TimeoutCommand {
  fn bot(&self) -> Arc<LalafellBot> {
    self.bot.clone()
  }
}

impl<'a> PublicChannelCommand<'a> for TimeoutCommand {
  fn run(&self, message: &Message, channel: &PublicChannel, params: &[&str]) -> CommandResult<'a> {
    let server_id = channel.server_id;
    let state_option = self.bot.state.read().unwrap();
    let state = state_option.as_ref().unwrap();
    let server = match state.servers().iter().find(|x| x.id == server_id) {
      Some(s) => s,
      None => {
        let err: error::Error = "could not find server for channel".into();
        return Err(err.into());
      }
    };
    let can_manage_roles = server.permissions_for(channel.id, message.author.id).contains(permissions::MANAGE_ROLES);
    if !can_manage_roles {
      return Err(ExternalCommandFailure::default()
        .message(|e: EmbedBuilder| e
          .title("Not enough permissions.")
          .description("You don't have enough permissions to use this command."))
        .wrap());
    }

    if params.len() < 2 {
      return Err(ExternalCommandFailure::default()
        .message(|e: EmbedBuilder| e
          .title("Not enough parameters.")
          .description(USAGE))
        .wrap());
    }
    let server_id = channel.server_id;
    let who = params[0];
    let who = if !who.starts_with("<@") && !who.ends_with('>') && message.mentions.len() != 1 {
      who.parse::<u64>().map(UserId).map_err(|_| ExternalCommandFailure::default()
        .message(|e: EmbedBuilder| e
          .title("Invalid target.")
          .description("The target was not a mention, and it was not a user ID."))
        .wrap())?
    } else {
      message.mentions[0].id
    };

    let role_id = {
      let state_option = self.bot.state.read().unwrap();
      let state = state_option.as_ref().unwrap();
      let live_server = match state.servers().iter().find(|s| s.id == server_id) {
        Some(s) => s,
        None => return Err("Could not find the server in the bot state. This is a bug.".into())
      };

      match timeout::set_up_timeouts(self.bot.as_ref(), live_server) {
        Ok(r) => {
          if let Err(e) = self.bot.discord.add_user_to_role(live_server.id, who, r) {
            warn!("could not add user {} to timeout role: {}", who.0, e);
          }
          r
        },
        Err(e) => {
          warn!("could not set up timeouts for {}: {}", live_server.id.0, e);
          return Err("Could not set up timeouts for this server. Do I have enough permissions?".into());
        }
      }
    };

    let duration = match TimeoutCommand::parse_duration(&params[1..].to_vec().into_iter().collect::<String>()) {
      Ok(d) => d,
      Err(_) => return Err("Invalid time length. Try \"15m\" or \"3 hours\" for example.".into())
    };

    let mut database = self.bot.database.write().unwrap();
    if database.timeouts.iter().any(|u| u.user_id == who.0 && u.server_id == server_id.0) {
      return Err(format!("{} is already timed out.", who.mention()).into());
    }

    let timeout_user = TimeoutUser::new(server_id.0, who.0, role_id.0, duration as i64, UTC::now().timestamp());
    database.timeouts.push(timeout_user);
    Ok(CommandSuccess::default())
  }
}
