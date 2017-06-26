use bot::LalafellBot;
use commands::*;
use error::*;

use discord::builders::EmbedBuilder;
use discord::model::{LiveServer, PublicChannel, UserId, RoleId};
use discord::model::permissions;

use std::sync::Arc;

const USAGE: &'static str = "!untimeout <who>";

pub struct UntimeoutCommand {
  bot: Arc<LalafellBot>
}

impl UntimeoutCommand {
  pub fn new(bot: Arc<LalafellBot>) -> UntimeoutCommand {
    UntimeoutCommand {
      bot: bot
    }
  }
}

impl HasBot for UntimeoutCommand {
  fn bot(&self) -> Arc<LalafellBot> {
    self.bot.clone()
  }
}

#[derive(Debug, Deserialize)]
pub struct Params {
  who: String
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
    let who = if !who.starts_with("<@") && !who.ends_with('>') && message.mentions.len() != 1 {
      who.parse::<u64>().map(UserId).map_err(|_| ExternalCommandFailure::default()
        .message(|e: EmbedBuilder| e
          .title("Invalid target.")
          .description("The target was not a mention, and it was not a user ID."))
        .wrap())?
    } else {
      message.mentions[0].id
    };

    let mut database = self.bot.database.write().unwrap();
    let timeout = match database.timeouts.iter().position(|u| u.user_id == who.0 && u.server_id == server_id.0) {
      Some(i) => database.timeouts.remove(i),
      None => return Err(format!("{} is not timed out.", who.mention()).into())
    };

    self.bot.discord.remove_user_from_role(server_id, who, RoleId(timeout.role_id)).chain_err(|| "could not remove timeout role")?;

    Ok(CommandSuccess::default())
  }
}
