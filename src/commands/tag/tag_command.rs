use bot::BotEnv;
use commands::*;
use commands::tag::Tagger;

use lalafell::error::*;
use lalafell::commands::prelude::*;

use serenity::builder::CreateEmbed;

const USAGE: &str = "!tag <who> <server> <character>";

pub struct TagCommand {
  env: Arc<BotEnv>
}

impl BotCommand for TagCommand {
  fn new(env: Arc<BotEnv>) -> Self {
    TagCommand { env }
  }
}

#[derive(Debug, Deserialize)]
pub struct Params {
  who: MentionOrId,
  server: String,
  name: [String; 2]
}

impl HasParams for TagCommand {
  type Params = Params;
}

impl<'a> PublicChannelCommand<'a> for TagCommand {
  fn run(&self, _: &Context, message: &Message, guild: GuildId, _: Arc<RwLock<GuildChannel>>, params: &[&str]) -> CommandResult<'a> {
    let params = self.params(USAGE, params)?;
    let member = guild.member(&message.author).chain_err(|| "could not get member")?;
    if !member.permissions().chain_err(|| "could not get permissions")?.manage_roles() {
      return Err(ExternalCommandFailure::default()
        .message(|e: CreateEmbed| e
          .title("Not enough permissions.")
          .description("You don't have enough permissions to use this command."))
        .wrap());
    }

    let who = params.who;
    let ff_server = params.server;
    let name = params.name.join(" ");

    match Tagger::search_tag(self.env.as_ref(), *who, guild, &ff_server, &name, true) {
      Ok(Some(error)) => Err(ExternalCommandFailure::default()
        .message(move |e: CreateEmbed| e.description(&error))
        .wrap()),
      Ok(None) => Ok(CommandSuccess::default()),
      Err(_) => Err(ExternalCommandFailure::default()
        .message(move |e: CreateEmbed| e.description("There was an error while tagging. The user most likely does not exist or is not on the server."))
        .wrap())
    }
  }
}
