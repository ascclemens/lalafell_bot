mod add;
mod list;
mod remove;

use lalafell::commands::prelude::*;
use lalafell::error::*;

use serenity::model::id::{GuildId, UserId};

#[derive(Debug, StructOpt)]
pub enum Params {
  #[structopt(name = "add", alias = "create", about = "Add a reaction role")]
  #[structopt(raw(template = "::lalafell::commands::TEMPLATE"))]
  Add(add::Params),

  #[structopt(name = "remove", alias = "delete", about = "Remove a reaction role")]
  #[structopt(raw(template = "::lalafell::commands::TEMPLATE"))]
  Remove(remove::Params),

  #[structopt(name = "list", alias = "show", about = "List active reaction roles")]
  #[structopt(raw(template = "::lalafell::commands::TEMPLATE"))]
  List
}

pub struct ReactionCommand;

impl<'a> ReactionCommand {
  pub fn run(&self, author: UserId, guild: GuildId, params: Params) -> CommandResult<'a> {
    struct SubCommands {
      add: add::AddCommand,
      remove: remove::RemoveCommand,
      list: list::ListCommand
    }

    const SUBCOMMANDS: SubCommands = SubCommands {
      add: add::AddCommand,
      remove: remove::RemoveCommand,
      list: list::ListCommand
    };

    let member = guild.member(author).chain_err(|| "could not get member")?;
    if !member.permissions().chain_err(|| "could not get permissions")?.manage_roles() {
      return Err(ExternalCommandFailure::default()
        .message(|e: CreateEmbed| e
          .title("Not enough permissions.")
          .description("You don't have enough permissions to use this command."))
        .wrap());
    }

    match params {
      Params::Add(p) => SUBCOMMANDS.add.run(guild, p),
      Params::Remove(p) => SUBCOMMANDS.remove.run(guild, p),
      Params::List => SUBCOMMANDS.list.run(guild)
    }
  }
}
