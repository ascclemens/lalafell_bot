use bot::BotEnv;
use commands::tag::Tagger;

use lalafell::commands::prelude::*;

use serenity::builder::CreateEmbed;

const USAGE: &str = "!autotag <server> <character>";

pub struct AutoTagCommand {
  env: Arc<BotEnv>
}

impl AutoTagCommand {
  pub fn new(env: Arc<BotEnv>) -> AutoTagCommand {
    AutoTagCommand { env }
  }
}

#[derive(Debug, Deserialize)]
pub struct Params {
  server: String,
  name: [String; 2]
}

impl HasParams for AutoTagCommand {
  type Params = Params;
}

impl<'a> PublicChannelCommand<'a> for AutoTagCommand {
  fn run(&self, _: &Context, message: &Message, guild: GuildId, _: Arc<RwLock<GuildChannel>>, params: &[&str]) -> CommandResult<'a> {
    let params = self.params(USAGE, params)?;
    let ff_server = params.server;
    let name = params.name.join(" ");

    match Tagger::search_tag(self.env.as_ref(), message.author.id, guild, &ff_server, &name, false)? {
      Some(error) => Err(ExternalCommandFailure::default()
        .message(move |e: CreateEmbed| e.description(&error))
        .wrap()),
      None => Ok(CommandSuccess::default())
    }
  }
}
