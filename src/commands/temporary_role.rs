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
    requires = "messages"
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
    let member = guild_id.member(&msg.author).chain_err(|| "could not get member")?;
    if !member.permissions().chain_err(|| "could not get permissions")?.manage_roles() {
      return Err(ExternalCommandFailure::default()
        .message(|e: CreateEmbed| e
          .title("Not enough permissions.")
          .description("You don't have enough permissions to use this command."))
        .wrap());
    }

    let params = self.params_then("temporaryrole", params, |a| a
      .setting(::structopt::clap::AppSettings::ArgRequiredElseHelp)
      .group(ArgGroup::with_name("m_or_t")
        .args(&["messages", "time"])
        .multiple(true)
        .required(true)))?;

    let guild = guild_id.to_guild_cached().chain_err(|| "could not find guild in cache")?;

    let mut target = match guild_id.member(*params.who) {
      Ok(m) => m,
      Err(_) => return Err("That person is not in this guild.".into())
    };

    let role_name = UniCase::new(params.role);
    let role = match guild.read().roles.values().find(|r| UniCase::new(r.name.as_str()) == role_name) {
      Some(r) => r.id,
      None => return Err("No such role.".into())
    };

    let messages = params.messages.map(|m| m as i32);
    let time = params.time
      .as_ref()
      .map(|t| Utc::now() + Duration::seconds(**t as i64))
      .map(|t| t.timestamp());
    let ntr = NewTemporaryRole::new(guild_id.0, params.who.0, role.0, msg.id.0, params.channel.map(|x| x.0), messages, time);
    let temp_role = ::bot::with_connection(|c| {
      use database::schema::temporary_roles::dsl;

      diesel::insert_into(dsl::temporary_roles).values(&ntr).get_result(c)
    }).chain_err(|| "could not store new temporary role")?;

    if let Some(t) = params.time {
      if *t < 600 {
        ::std::thread::spawn(move || {
          ::std::thread::sleep(Duration::seconds(*t as i64).to_std().unwrap());
          ::tasks::temporary_roles::remove_temporary_role(&temp_role);
        });
      }
    }

    target.add_role(role).chain_err(|| "could not add role")?;
    Ok(CommandSuccess::default())
  }
}
