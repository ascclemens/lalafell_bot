use bot::BotEnv;

use lalafell::commands::prelude::*;

use serenity::model::gateway::{Game, GameType};

const USAGE: &str = "!presence <type/\"random\"> <content>";

pub struct PresenceCommand {
  env: Arc<BotEnv>
}

impl PresenceCommand {
  pub fn new(env: Arc<BotEnv>) -> Self {
    PresenceCommand { env }
  }
}

#[derive(Debug, Deserialize)]
pub struct Params {
  kind: String,
  content: Option<Vec<String>>
}

impl HasParams for PresenceCommand {
  type Params = Params;
}

impl<'a> Command<'a> for PresenceCommand {
  fn run(&self, ctx: &Context, msg: &Message, params: &[&str]) -> CommandResult<'a> {
    if !self.env.config.read().bot.administrators.contains(&msg.author.id.0) {
      return Err(ExternalCommandFailure::default()
        .message(|e: CreateEmbed| e
          .title("Not enough permissions.")
          .description("You don't have enough permissions to use this command."))
        .wrap());
    }
    let params = self.params(USAGE, params)?;
    let game = if params.kind.to_lowercase() == "random" {
      match ::tasks::random_presence::random_game(self.env.as_ref()) {
        Some(g) => g,
        None => return Err("No presences.".into())
      }
    } else {
      if params.content.is_none() {
        return Err("You must specify a game name.".into());
      }
      let game_type = match params.kind.as_str() {
        "playing" => GameType::Playing,
        "streaming" => GameType::Streaming,
        "listening" => GameType::Listening,
        _ => return Err("Invalid presence type.".into())
      };
      Game {
        kind: game_type,
        name: params.content.clone().unwrap().join(" "),
        url: None
      }
    };
    ctx.set_game(game);
    Ok(CommandSuccess::default())
  }
}
