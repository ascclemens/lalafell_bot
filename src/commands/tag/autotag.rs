use bot::BotEnv;
use commands::tag::Tagger;

use ffxiv::World;

use lalafell::commands::prelude::*;

use serenity::builder::CreateEmbed;

#[derive(BotCommand)]
pub struct AutoTagCommand {
  env: Arc<BotEnv>
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Tag yourself as a FFXIV character")]
pub struct Params {
  #[structopt(help = "The server the character is on, e.g. \"Adamantoise\"")]
  server: World,
  #[structopt(help = "The first name of the character")]
  first_name: String,
  #[structopt(help = "The last name of the character")]
  last_name: String
}

impl HasParams for AutoTagCommand {
  type Params = Params;
}

impl<'a> PublicChannelCommand<'a> for AutoTagCommand {
  fn run(&self, _: &Context, message: &Message, guild: GuildId, _: Arc<RwLock<GuildChannel>>, params: &[&str]) -> CommandResult<'a> {
    let params = self.params_then("autotag", params, |a| a.setting(::structopt::clap::AppSettings::ArgRequiredElseHelp))?;
    let ff_server = params.server;
    let name = format!("{} {}", params.first_name, params.last_name);

    match Tagger::search_tag(self.env.as_ref(), message.author.id, guild, ff_server.as_str(), &name, false)? {
      Some(error) => Err(ExternalCommandFailure::default()
        .message(move |e: CreateEmbed| e.description(&error))
        .wrap()),
      None => Ok(CommandSuccess::default())
    }
  }
}
