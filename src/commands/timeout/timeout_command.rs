use crate::commands::*;
use crate::database::models::{ToU64, NewTimeout};
use crate::database::schema::timeouts;
use crate::util::parse_duration_secs;

use lalafell::error::*;
use lalafell::commands::prelude::*;

use serenity::builder::CreateEmbed;
use serenity::model::channel::{Message, GuildChannel};
use serenity::model::misc::Mentionable;

use diesel::prelude::*;

use chrono::prelude::*;
use chrono::Duration;

use std::sync::Arc;

#[derive(BotCommand)]
pub struct TimeoutCommand {
  env: Arc<BotEnv>,
}

#[derive(Debug, StructOpt)]
#[structopt(help = "Put a member in time out, preventing them from doing anything")]
pub struct Params {
  #[structopt(help = "Who to timeout")]
  who: MentionOrId,
  #[structopt(help = "How long to time out the person for")]
  length: Vec<String>
}

impl HasParams for TimeoutCommand {
  type Params = Params;
}

impl<'a> PublicChannelCommand<'a> for TimeoutCommand {
  fn run(&self, ctx: &Context, message: &Message, guild: GuildId, channel: Arc<RwLock<GuildChannel>>, params: &[&str]) -> CommandResult<'a> {
    let params = self.params_then("timeout", params, |a| a.setting(structopt::clap::AppSettings::ArgRequiredElseHelp))?;
    let member = guild.member(ctx, &message.author).chain_err(|| "could not get member")?;
    if !member.permissions(&ctx).chain_err(|| "could not get permissions")?.manage_roles() {
      return Err(ExternalCommandFailure::default()
        .message(|e: &mut CreateEmbed| e
          .title("Not enough permissions.")
          .description("You don't have enough permissions to use this command."))
        .wrap());
    }

    let server_id = channel.read().guild_id;
    let who = params.who;

    let mut timeout_member = guild.member(ctx, *who).map_err(|_| into!(CommandFailure, "That user is not in this guild."))?;

    let guild = guild.to_guild_cached(&ctx).chain_err(|| "could not find guild")?;

    let timeouts = crate::bot::with_connection(|c| {
      use crate::database::schema::timeouts::dsl;
      use diesel::expression::dsl::count;
      dsl::timeouts
        .filter(dsl::user_id.eq(who.to_u64()).and(dsl::server_id.eq(server_id.to_u64())))
        .select(count(dsl::id))
        .first(c)
        .optional()
    }).chain_err(|| "could not load timeouts")?;
    if timeouts.unwrap_or(0) > 0 {
      return Err(format!("{} is already timed out.", who.mention()).into());
    }

    let role_id = match timeout::set_up_timeouts(ctx, &guild.read()) {
      Ok(r) => {
        if let Err(e) = timeout_member.add_role(&ctx, r) {
          warn!("could not add user {} to timeout role: {}", who.0, e);
        }
        r
      },
      Err(e) => {
        warn!("could not set up timeouts for {}: {}", guild.read().id.0, e);
        return Err("Could not set up timeouts for this server. Do I have enough permissions?".into());
      }
    };

    let duration = match parse_duration_secs(&params.length.into_iter().collect::<String>()) {
      Ok(d) => d,
      Err(_) => return Err("Invalid time length. Try \"15m\" or \"3 hours\" for example.".into())
    };

    let timeout_user = NewTimeout::new(who.0, server_id.0, role_id.0, duration as i32, Utc::now().timestamp());
    let timeout = crate::bot::with_connection(|c| diesel::insert_into(timeouts::table).values(&timeout_user).get_result(c)).chain_err(|| "could not insert timeout")?;

    // spawn a task if the duration is less than the check task period
    if duration < 300 {
      let env = Arc::clone(&self.env);
      std::thread::spawn(move || {
        std::thread::sleep(Duration::seconds(duration as i64).to_std().unwrap());
        crate::tasks::timeout_check::remove_timeout(&env, &timeout);
      });
    }

    Ok(CommandSuccess::default())
  }
}
