use lalafell::error::*;
use lalafell::commands::prelude::*;
use lalafell::commands::MentionOrId;

use rand::{Rng, thread_rng};

use serenity::model::channel::{ChannelType, PermissionOverwrite, PermissionOverwriteType};
use serenity::model::permissions::Permissions;

use unicase::UniCase;

use std::sync::Arc;

#[derive(BotCommand)]
pub struct ReportCommand;

#[derive(Debug, StructOpt)]
#[structopt(about = "Create a private channel with a member")]
pub struct Params {
  #[structopt(help = "The member to assign the role to")]
  who: MentionOrId,
}

impl HasParams for ReportCommand {
  type Params = Params;
}

impl<'a> PublicChannelCommand<'a> for ReportCommand {
  fn run(&self, _: &Context, msg: &Message, guild_id: GuildId, _: Arc<RwLock<GuildChannel>>, params: &[&str]) -> CommandResult<'a> {
    let member = guild_id.member(&msg.author).chain_err(|| "could not get member")?;
    if !member.permissions().chain_err(|| "could not get permissions")?.manage_roles() {
      return Err(ExternalCommandFailure::default()
        .message(|e: CreateEmbed| e
          .title("Not enough permissions.")
          .description("You don't have enough permissions to use this command."))
        .wrap());
    }

    let params = self.params("report", params)?;

    let guild = guild_id.find().chain_err(|| "could not find guild in cache")?;

    let reports_name = UniCase::new("Reports");
    let category = guild.read().channels
      .iter()
      .find(|(_, x)| {
        let channel = x.read();
        channel.kind == ChannelType::Category && UniCase::new(channel.name.as_str()) == reports_name
      })
      .map(|(&x, _)| x);

    let everyone = match guild.read().roles.values().find(|r| r.name == "@everyone") {
      Some(r) => r.id,
      None => return Err("No `@everyone` role?".into()),
    };
    let moderator_name = UniCase::new("moderator");
    let moderator = match guild.read().roles.values().find(|r| UniCase::new(r.name.as_str()) == moderator_name) {
        Some(r) => r.id,
        None => return Err("No `moderator` role.".into()),
      };

    let deny_everyone = PermissionOverwrite {
      allow: Permissions::empty(),
      deny: Permissions::READ_MESSAGES,
      kind: PermissionOverwriteType::Role(everyone),
    };

    let allow_moderators = PermissionOverwrite {
      allow: Permissions::READ_MESSAGES,
      deny: Permissions::empty(),
      kind: PermissionOverwriteType::Role(moderator),
    };

    let allow_reporter = PermissionOverwrite {
      allow: Permissions::READ_MESSAGES,
      deny: Permissions::empty(),
      kind: PermissionOverwriteType::Member(*params.who),
    };

    let chars: String = thread_rng().gen_ascii_chars().take(7).collect();
    let channel_name = format!("report_{}", chars);
    let channel = guild_id
      .create_channel(&channel_name, ChannelType::Text, category)
      .chain_err(|| "could not create channel")?;

    channel.create_permission(&deny_everyone).chain_err(|| "could not deny @everyone")?;
    channel.create_permission(&allow_moderators).chain_err(|| "could not allow moderators")?;
    channel.create_permission(&allow_reporter).chain_err(|| "could not allow reporter")?;

    Ok(CommandSuccess::default())
  }
}
