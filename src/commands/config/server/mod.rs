pub mod auto_reply;
pub mod delete_all_messages;
pub mod reaction;
pub mod timeout_role;

use lalafell::commands::prelude::*;

use serenity::model::id::{UserId, GuildId};

#[derive(Default)]
pub struct ServerCommand;

impl<'a> ServerCommand {
  pub fn run(&self, ctx: &Context, author: UserId, guild: GuildId, params: Params) -> CommandResult<'a> {
    struct SubCommands {
      auto_reply: auto_reply::AutoReplyCommand,
      delete_all_messages: delete_all_messages::DeleteAllMessagesCommand,
      reaction: reaction::ReactionCommand,
      timeout_role: timeout_role::TimeoutRoleCommand
    }

    const SUBCOMMANDS: SubCommands = SubCommands {
      auto_reply: auto_reply::AutoReplyCommand,
      delete_all_messages: delete_all_messages::DeleteAllMessagesCommand,
      reaction: reaction::ReactionCommand,
      timeout_role: timeout_role::TimeoutRoleCommand
    };

    match params {
      Params::AutoReply(p) => SUBCOMMANDS.auto_reply.run(ctx, author, guild, p),
      Params::DeleteAllMessages(p) => SUBCOMMANDS.delete_all_messages.run(ctx, author, guild, p),
      Params::Reaction(p) => SUBCOMMANDS.reaction.run(ctx, author, guild, p),
      Params::TimeoutRole(p) => SUBCOMMANDS.timeout_role.run(ctx, author, guild, p)
    }
  }
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Configure server settings.")]
pub enum Params {
  #[structopt(name = "autoreply", alias = "ar", about = "Manage auto-reply settings")]
  #[structopt(raw(setting = "::structopt::clap::AppSettings::ArgRequiredElseHelp"))]
  #[structopt(raw(template = "::lalafell::commands::TEMPLATE"))]
  AutoReply(auto_reply::Params),

  #[structopt(name = "deleteallmessages", alias = "dam", about = "Manage channel message deletion settings")]
  #[structopt(raw(aliases = r#"&["dam", "dams"]"#))]
  #[structopt(raw(setting = "::structopt::clap::AppSettings::ArgRequiredElseHelp"))]
  #[structopt(raw(template = "::lalafell::commands::TEMPLATE"))]
  DeleteAllMessages(delete_all_messages::Params),

  #[structopt(name = "reaction", alias = "reactions", about = "Manage reaction role settings")]
  #[structopt(raw(setting = "::structopt::clap::AppSettings::ArgRequiredElseHelp"))]
  #[structopt(raw(template = "::lalafell::commands::TEMPLATE"))]
  Reaction(reaction::Params),

  #[structopt(name = "timeoutrole", about = "Manage the timeout role")]
  #[structopt(raw(template = "::lalafell::commands::TEMPLATE"))]
  TimeoutRole(timeout_role::Params)
}
