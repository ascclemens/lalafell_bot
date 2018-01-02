use bot::LalafellBot;
use database::models::Timeout;
use commands::*;

use lalafell::error::*;
use lalafell::bot::Bot;
use lalafell::commands::prelude::*;

use discord::builders::EmbedBuilder;
use discord::model::{Message, LiveServer, PublicChannel, RoleId};
use discord::model::permissions;

use diesel::prelude::*;

use std::sync::Arc;

const USAGE: &'static str = "!untimeout <who>";

pub struct UntimeoutCommand {
  bot: Arc<LalafellBot>
}

impl UntimeoutCommand {
  pub fn new(bot: Arc<LalafellBot>) -> UntimeoutCommand {
    UntimeoutCommand {
      bot
    }
  }
}

impl HasBot for UntimeoutCommand {
  fn bot(&self) -> &Bot {
    self.bot.as_ref()
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
  fn run(&self, message: &Message, server: &LiveServer, channel: &PublicChannel, params: &[&str]) -> CommandResult<'a> {
    let params = self.params(USAGE, params)?;
    let can_manage_roles = server.permissions_for(channel.id, message.author.id).contains(permissions::MANAGE_ROLES);
    if !can_manage_roles {
      return Err(ExternalCommandFailure::default()
        .message(|e: EmbedBuilder| e
          .title("Not enough permissions.")
          .description("You don't have enough permissions to use this command."))
        .wrap());
    }

    let server_id = channel.server_id;
    let who = params.who;

    let timeouts: Vec<Timeout> = ::bot::CONNECTION.with(|c| {
      use database::schema::timeouts::dsl;
      dsl::timeouts
        .filter(dsl::user_id.eq(who.0.to_string()).and(dsl::server_id.eq(server_id.0.to_string())))
        .load(c)
        .chain_err(|| "could not load timeouts")
    })?;
    if timeouts.is_empty() {
      return Err("That user is not timed out.".into());
    }
    let timeout = &timeouts[0];

    ::bot::CONNECTION.with(|c| ::diesel::delete(timeout).execute(c).chain_err(|| "could not delete timeout"))?;
    self.bot.discord.remove_user_from_role(server_id, *who, RoleId(*timeout.role_id)).chain_err(|| "could not remove timeout role")?;

    Ok(CommandSuccess::default())
  }
}
