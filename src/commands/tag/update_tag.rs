use bot::LalafellBot;
use commands::*;
use tasks::AutoTagTask;

use discord::model::{LiveServer, PublicChannel, UserId, ServerId, permissions};

use std::sync::Arc;

pub struct UpdateTagCommand {
  bot: Arc<LalafellBot>
}

impl UpdateTagCommand {
  pub fn new(bot: Arc<LalafellBot>) -> UpdateTagCommand {
    UpdateTagCommand {
      bot: bot
    }
  }
}

impl HasBot for UpdateTagCommand {
  fn bot(&self) -> Arc<LalafellBot> {
    self.bot.clone()
  }
}

impl<'a> PublicChannelCommand<'a> for UpdateTagCommand {
  fn run(&self, message: &Message, server: &LiveServer, channel: &PublicChannel, params: &[&str]) -> CommandResult<'a> {
    let id = if !params.is_empty() {
      let can_manage_roles = server.permissions_for(channel.id, message.author.id).contains(permissions::MANAGE_ROLES);
      if !can_manage_roles {
        return Err(ExternalCommandFailure::default()
          .message(|e: EmbedBuilder| e
            .title("Not enough permissions.")
            .description("You don't have enough permissions to update other people's tags."))
          .wrap());
      }
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
      who
    } else {
      message.author.id
    };
    let user: Option<(UserId, ServerId, u64)> = {
      let database = self.bot.database.read().unwrap();
      database.autotags.users.iter()
        .find(|u| u.user_id == id.0 && u.server_id == channel.server_id.0)
        .map(|u| (UserId(u.user_id), ServerId(u.server_id), u.character_id))
    };
    let (user_id, server_id, character_id) = match user {
      Some(u) => u,
      None => return if id == message.author.id {
        Err("You are not set up with a tag. Use `!autotag` to tag yourself.".into())
      } else {
        Err(format!("{} is not set up with a tag.", id.mention()).into())
      }
    };
    let option_state = self.bot.state.read().unwrap();
    let state = match option_state.as_ref() {
      Some(st) => st,
      None => return Err("I'm not fully synced with Discord! Please try again later.".into())
    };
    match AutoTagTask::update_tag(self.bot.as_ref(), state, user_id, server_id, character_id) {
      Ok(Some(err)) => Err(err.into()),
      Err(e) => Err(e.into()),
      Ok(None) => Ok(CommandSuccess::default())
    }
  }
}
