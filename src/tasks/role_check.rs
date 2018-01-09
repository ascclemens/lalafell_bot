use error::*;
use bot::BotEnv;
use tasks::RunsTask;
use util::parse_duration_secs;
use database::models::{RoleCheckTime, NewRoleCheckTime};

use diesel::prelude::*;

use chrono::{Utc, TimeZone, Duration};

use serenity::prelude::Mentionable;
use serenity::model::id::{GuildId, ChannelId, RoleId, UserId};
use serenity::model::guild::{Member, Role};

use serde_json;

use std::thread;
use std::sync::Arc;
use std::collections::HashMap;

macro_rules! config {
  ($env:expr) => {{
    match $env.config.read().bot.tasks.get("role_check").cloned().map(serde_json::from_value) {
      Some(Ok(rc)) => rc,
      Some(Err(e)) => {
        warn!("invalid role_check in config: {}", e);
        return;
      },
      None => {
        warn!("missing role_check in config");
        return;
      }
    }
  }}
}

pub struct RoleCheckTask {
  first_run: bool
}

impl RoleCheckTask {
  pub fn new() -> Self {
    RoleCheckTask { first_run: true }
  }
}

impl RunsTask for RoleCheckTask {
  fn start(mut self, env: Arc<BotEnv>) {
    loop {
      let config: RoleCheckConfig = config!(env.as_ref());
      let sleep = if self.first_run {
        self.first_run = false;
        config.delay
      } else {
        config.period
      };
      info!("Waiting {} second{}", sleep, if sleep == 1 { "" } else { "s" });
      thread::sleep(Duration::seconds(sleep).to_std().unwrap());

      let now = Utc::now();
      for check in config.checks {
        let reminder_secs = match parse_duration_secs(&check.reminder.time) {
          Ok(s) => s,
          Err(_) => {
            warn!("invalid reminder time: {}", check.reminder.time);
            continue;
          }
        };
        let kick_secs = match parse_duration_secs(&check.kick.time) {
          Ok(s) => s,
          Err(_) => {
            warn!("invalid kick time: {}", check.kick.time);
            continue;
          }
        };
        let guild = some_or!(GuildId(check.guild).find(), continue);
        let roles = guild.read().roles.clone();
        let times: Result<Vec<RoleCheckTime>> = ::bot::CONNECTION.with(|c| {
          use database::schema::role_check_times::dsl;
          dsl::role_check_times
            .filter(dsl::check_id.eq(check.id))
            .load(c)
            .chain_err(|| "could not load role_check_times")
        });
        let times = match times {
          Ok(t) => t,
          Err(e) => {
            warn!("{}", e);
            continue;
          }
        };
        let members: Vec<(UserId, Member)> = guild.read().members.iter()
          .filter(|&(_, m)| check.necessary_roles.matches(m, &roles))
          .map(|(id, m)| (*id, m.clone()))
          .collect();
        let (remove, times): (Vec<RoleCheckTime>, Vec<RoleCheckTime>) = times.into_iter()
          .partition(|t| members.iter().find(|&&(id, _)| id.0 == *t.user_id).is_none());
        ::bot::CONNECTION.with(|c| {
          for r in remove {
            if let Err(e) = ::diesel::delete(&r).execute(c) {
              warn!("Could not delete old role_check_time {}: {}", r.id, e);
            }
          }
        });
        let mut reminders = Vec::new();
        for (user_id, member) in members {
          match times.iter().find(|x| *x.user_id == user_id.0) {
            Some(time) => {
              if Utc.timestamp(time.reminded_at, 0) + Duration::seconds(i64::from(time.kick_after)) <= now {
                // TODO: use message when able
                info!("Kicking user {} ({}) on {} ({}) due to check {}",
                  member.display_name(),
                  user_id.0,
                  guild.read().name,
                  guild.read().id.0,
                  check.id);
                match member.kick() {
                  Ok(_) => ::bot::CONNECTION.with(|c| {
                    if let Err(e) = ::diesel::delete(time).execute(c) {
                      warn!("Could not remove database entry for check after kick: {}", e);
                    }
                  }),
                  Err(e) => warn!("Kick was not successful: {}", e)
                }
              }
            },
            None => if let Some(joined) = member.joined_at {
              if now.signed_duration_since(joined.with_timezone(&Utc)) >= Duration::seconds(reminder_secs as i64) {
                reminders.push(member.mention());
                let new_time = NewRoleCheckTime::new(check.id, user_id.0, now.timestamp(), kick_secs as i32);
                ::bot::CONNECTION.with(move |c| {
                  use database::schema::role_check_times::dsl;
                  ::diesel::insert_into(dsl::role_check_times)
                    .values(&new_time)
                    .execute(c)
                    .ok();
                });
              }
            }
          }
        }
        if reminders.is_empty() {
          continue;
        }
        let mentions = reminders.join(" ");
        if let Err(e) = ChannelId(check.channel).send_message(|m| m.content(check.reminder.message.replace("{mentions}", &mentions))) {
          warn!("Could not send reminder message for check {}: {}", check.id, e);
        }
      }
    }
  }
}

#[derive(Debug, Deserialize)]
struct RoleCheckConfig {
  delay: i64,
  period: i64,
  checks: Vec<RoleCheck>
}

#[derive(Debug, Deserialize)]
struct RoleCheck {
  id: i32,
  guild: u64,
  channel: u64,
  necessary_roles: NeededRole,
  reminder: RoleCheckMessage,
  kick: RoleCheckMessage
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum NeededRole {
  Simple(String),
  Logical(NeededRoleLogical)
}

impl NeededRole {
  fn matches(&self, member: &Member, roles: &HashMap<RoleId, Role>) -> bool {
    match *self {
      NeededRole::Simple(ref role) => roles.iter().find(|x| x.1.name.to_lowercase() == role.to_lowercase()).map(|r| member.roles.contains(r.0)).unwrap_or_default(),
      NeededRole::Logical(NeededRoleLogical::And(ref b)) => b.iter().all(|x| x.matches(member, roles)),
      NeededRole::Logical(NeededRoleLogical::Or(ref b)) => b.iter().any(|x| x.matches(member, roles)),
      NeededRole::Logical(NeededRoleLogical::Not(ref x)) => !x.matches(member, roles)
    }
  }
}

#[derive(Debug, Deserialize)]
enum NeededRoleLogical {
  #[serde(rename = "and")]
  And(Vec<Box<NeededRole>>),
  #[serde(rename = "or")]
  Or(Vec<Box<NeededRole>>),
  #[serde(rename = "not")]
  Not(Box<NeededRole>)
}

#[derive(Debug, Deserialize)]
struct RoleCheckMessage {
  time: String,
  message: String
}
