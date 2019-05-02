use crate::bot::BotEnv;
use crate::commands::*;
use crate::tasks::AutoTagTask;
use crate::database::models::{ToU64, Tag};

use serenity::{
  builder::CreateEmbed,
  model::id::{GuildId, UserId},
  prelude::Mentionable,
};

use lalafell::commands::prelude::*;
use lalafell::error::*;

use diesel::prelude::*;

#[derive(BotCommand)]
pub struct UpdateTagCommand {
  env: Arc<BotEnv>
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Update your own tag or someone else's right now")]
pub struct Params {
  #[structopt(help = "Who to update the tag for if not yourself (assuming you have permission)")]
  who: Option<MentionOrId>
}

impl HasParams for UpdateTagCommand {
  type Params = Params;
}

impl<'a> PublicChannelCommand<'a> for UpdateTagCommand {
  fn run(&self, ctx: &Context, message: &Message, guild: GuildId, _: Arc<RwLock<GuildChannel>>, params: &[&str]) -> CommandResult<'a> {
    let params = self.params("updatetag", params)?;
    let id = match params.who {
      Some(who) => {
        let member = guild.member(&ctx, &message.author).chain_err(|| "could not get member")?;
        if !member.permissions(&ctx).chain_err(|| "could not get permissions")?.manage_roles() {
          return Err(ExternalCommandFailure::default()
            .message(|e: &mut CreateEmbed| e
              .title("Not enough permissions.")
              .description("You don't have enough permissions to update other people's tags."))
            .wrap());
        }
        *who
      },
      None => message.author.id
    };
    let tag: Option<Tag> = crate::bot::with_connection(|c| {
      use crate::database::schema::tags::dsl;
      dsl::tags
        .filter(dsl::user_id.eq(id.to_u64()).and(dsl::server_id.eq(guild.to_u64())))
        .first(c)
        .optional()
    }).chain_err(|| "could not load tags")?;
    let tag = match tag {
      Some(u) => u,
      None => return if id == message.author.id {
        Err("You are not set up with a tag. Use `!autotag` to tag yourself.".into())
      } else {
        Err(format!("{} is not set up with a tag.", id.mention()).into())
      }
    };
    match AutoTagTask::update_tag(self.env.as_ref(), UserId(*tag.user_id), GuildId(*tag.server_id), *tag.character_id) {
      Ok(Some(err)) => Err(err.into()),
      Err(e) => Err(e.into()),
      Ok(None) => Ok(CommandSuccess::default())
    }
  }
}
