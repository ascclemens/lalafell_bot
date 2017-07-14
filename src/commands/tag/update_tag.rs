use bot::LalafellBot;
use commands::*;
use tasks::AutoTagTask;
use database::models::Tag;

use lalafell::bot::Bot;
use lalafell::commands::prelude::*;
use lalafell::error::*;

use discord::model::{Message, LiveServer, PublicChannel, UserId, ServerId, permissions};

use diesel::prelude::*;

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
  fn bot(&self) -> &Bot {
    self.bot.as_ref()
  }
}

#[derive(Debug, Deserialize)]
pub struct Params {
  who: Option<MentionOrId>
}

impl HasParams for UpdateTagCommand {
  type Params = Params;
}

impl<'a> PublicChannelCommand<'a> for UpdateTagCommand {
  fn run(&self, message: &Message, server: &LiveServer, channel: &PublicChannel, params: &[&str]) -> CommandResult<'a> {
    let params = self.params("", params)?;
    let id = match params.who {
      Some(who) => {
        let can_manage_roles = server.permissions_for(channel.id, message.author.id).contains(permissions::MANAGE_ROLES);
        if !can_manage_roles {
          return Err(ExternalCommandFailure::default()
            .message(|e: EmbedBuilder| e
              .title("Not enough permissions.")
              .description("You don't have enough permissions to update other people's tags."))
            .wrap());
        }
        *who
      },
      None => message.author.id
    };
    let tag: Option<Tag> = ::bot::CONNECTION.with(|c| {
      use database::schema::tags::dsl;
      dsl::tags
        .filter(dsl::user_id.eq(id.0 as f64).and(dsl::server_id.eq(channel.server_id.0 as f64)))
        .first(c)
        .optional()
        .chain_err(|| "could not load tags")
    })?;
    let tag = match tag {
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
    match AutoTagTask::update_tag(self.bot.as_ref(), state, UserId(tag.user_id as u64), ServerId(tag.server_id as u64), tag.character_id as u64) {
      Ok(Some(err)) => Err(err.into()),
      Err(e) => Err(e.into()),
      Ok(None) => Ok(CommandSuccess::default())
    }
  }
}
