use database::models::NewTemporaryRole;
use util::ParsedDuration;

use chrono::{Utc, Duration};

use diesel;
use diesel::prelude::*;

use lalafell::error::*;
use lalafell::commands::prelude::*;
use lalafell::commands::{ChannelOrId, MentionOrId};

use structopt::clap::ArgGroup;

use unicase::UniCase;

use std::sync::Arc;

#[derive(BotCommand)]
pub struct TemporaryRoleCommand;

#[derive(Debug, StructOpt)]
#[structopt(about = "Give a role to a member for a given number of messages or time")]
pub struct Params {
  #[structopt(
    short = "w",
    long = "who",
    help = "The member to assign the role to"
  )]
  who: MentionOrId,
  #[structopt(
    short = "r",
    long = "role",
    help = "The role to assign to the member"
  )]
  role: String,
  #[structopt(
    short = "m",
    long = "messages",
    help = "The amount of messages for the role to last"
  )]
  messages: Option<u32>,
  #[structopt(
    short = "c",
    long = "channel",
    help = "The channel to count messages against, if any",
    conflicts_with = "time"
  )]
  channel: Option<ChannelOrId>,
  #[structopt(
    short = "t",
    long = "time",
    help = "The amount of time for the role to last"
  )]
  time: Option<ParsedDuration>
}

impl HasParams for TemporaryRoleCommand {
  type Params = Params;
}

impl<'a> PublicChannelCommand<'a> for TemporaryRoleCommand {
  fn run(&self, _: &Context, msg: &Message, guild_id: GuildId, _: Arc<RwLock<GuildChannel>>, params: &[&str]) -> CommandResult<'a> {
    let params = self.params_then("temporaryrole", params, |a| a
      .setting(::structopt::clap::AppSettings::ArgRequiredElseHelp)
      .group(ArgGroup::with_name("m_or_t")
        .args(&["messages", "time"])
        .required(true)))?;

    let guild = guild_id.find().chain_err(|| "could not find guild in cache")?;
    let mut member = guild.read().member(*params.who).chain_err(|| "could not find member")?;

    let role_name = UniCase::new(params.role);
    let role = match guild.read().roles.values().find(|r| UniCase::new(r.name.as_str()) == role_name) {
      Some(r) => r.id,
      None => return Err("No such role.".into())
    };

    let messages = params.messages.map(|m| m as i32);
    let time = params.time
      .map(|t| Utc::now() + Duration::seconds(*t as i64))
      .map(|t| t.timestamp());
    let ntr = NewTemporaryRole::new(guild_id.0, params.who.0, role.0, msg.id.0, params.channel.map(|x| x.0), messages, time);
    ::bot::with_connection(|c| {
      use database::schema::temporary_roles::dsl;

      diesel::insert_into(dsl::temporary_roles).values(&ntr).execute(c)
    }).chain_err(|| "could not store new temporary role")?;

    member.add_role(role).chain_err(|| "could not add role")?;
    Ok(CommandSuccess::default())
  }
}
