use bot::BotEnv;
use database::models::Timeout;
use commands::*;

use lalafell::error::*;
use lalafell::commands::prelude::*;

use serenity::builder::CreateEmbed;

use diesel::prelude::*;

use std::sync::Arc;

const USAGE: &str = "!untimeout <who>";

pub struct UntimeoutCommand;

impl UntimeoutCommand {
  pub fn new(_: Arc<BotEnv>) -> Self {
    UntimeoutCommand
  }
}

#[derive(Debug, Deserialize)]
pub struct Params {
  who: MentionOrId
}

impl HasParams for UntimeoutCommand {
  type Params = Params;
}

impl<'a> PublicChannelCommand<'a> for UntimeoutCommand {
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

    let guild_id = guild;
    let who = params.who;

    let mut timeout_member = match guild.member(*who) {
      Ok(m) => m,
      Err(_) => return Err("That user is not in this guild.".into())
    };

    let timeouts: Vec<Timeout> = ::bot::CONNECTION.with(|c| {
      use database::schema::timeouts::dsl;
      dsl::timeouts
        .filter(dsl::user_id.eq(who.0.to_string()).and(dsl::server_id.eq(guild_id.0.to_string())))
        .load(c)
        .chain_err(|| "could not load timeouts")
    })?;
    if timeouts.is_empty() {
      return Err("That user is not timed out.".into());
    }
    let timeout = &timeouts[0];

    ::bot::CONNECTION.with(|c| ::diesel::delete(timeout).execute(c).chain_err(|| "could not delete timeout"))?;
    timeout_member.remove_role(*timeout.role_id).chain_err(|| "could not remove role")?;

    Ok(CommandSuccess::default())
  }
}
