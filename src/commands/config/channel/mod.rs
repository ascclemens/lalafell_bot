pub mod image_dump;

use lalafell::commands::prelude::*;
use lalafell::commands::ChannelOrId;

use serenity::model::id::{UserId, GuildId};

#[derive(Default)]
pub struct ChannelCommand;

impl<'a> ChannelCommand {
  pub fn run(&self, ctx: &Context, author: UserId, guild: GuildId, params: Params) -> CommandResult<'a> {
    struct SubCommands {
      image_dump: image_dump::ImageDumpCommand
    }

    const SUBCOMMANDS: SubCommands = SubCommands {
      image_dump: image_dump::ImageDumpCommand
    };

    match params.subcommand {
      SubCommand::ImageDump(p) => SUBCOMMANDS.image_dump.run(ctx, author, guild, *params.channel, p)
    }
  }
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Configure channel settings.")]
pub struct Params {
  #[structopt(help = "The channel to configure")]
  channel: ChannelOrId,
  #[structopt(subcommand)]
  subcommand: SubCommand
}

#[derive(Debug, StructOpt)]
pub enum SubCommand {
  #[structopt(name = "imagedump", alias = "dump", about = "Manage !imagedump settings")]
  #[structopt(raw(template = "::lalafell::commands::TEMPLATE"))]
  ImageDump(image_dump::Params)
}
