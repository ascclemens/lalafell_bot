use bot::LalafellBot;
use commands::*;
use tasks::AutoTagTask;

use discord::model::{PublicChannel, UserId, ServerId};

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
  fn run(&self, message: &Message, channel: &PublicChannel, _: &[&str]) -> CommandResult<'a> {
    let user: Option<(UserId, ServerId, u64)> = {
      let database = self.bot.database.read().unwrap();
      database.autotags.users.iter()
        .find(|u| u.user_id == message.author.id.0 && u.server_id == channel.server_id.0)
        .map(|u| (UserId(u.user_id), ServerId(u.server_id), u.character_id))
    };
    let (user_id, server_id, character_id) = match user {
      Some(u) => u,
      None => return Err("You are not set up with a tag. Use `!autotag` to tag yourself.".into())
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