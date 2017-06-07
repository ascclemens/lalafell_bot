use LalafellBot;
use commands::*;

use discord::builders::EmbedBuilder;

use error::*;

use std::sync::Arc;

const USAGE: &'static str = "!race <server> <character>";

pub struct RaceCommand {
  bot: Arc<LalafellBot>
}

impl RaceCommand {
  pub fn new(bot: Arc<LalafellBot>) -> RaceCommand {
    RaceCommand {
      bot: bot
    }
  }
}

impl<'a> Command<'a> for RaceCommand {
  fn run(&self, _: &Message, params: &[&str]) -> CommandResult<'a> {
    if params.len() < 3 {
      return Err(ExternalCommandFailure::default()
        .message(|e: EmbedBuilder| e
          .title("Not enough parameters.")
          .description(USAGE))
        .wrap());
    }
    let server = params[0];
    let name = params[1..].join(" ");
    let params = &[
      ("one", "characters"),
      ("strict", "on"),
      ("server|et", server)
    ];
    let res = self.bot.xivdb.search(&name, params).chain_err(|| "could not search XIVDB")?;
    let search_chars = match res.characters {
      Some(c) => c.results,
      None => {
        let err: error::Error = "no characters field in search result".into();
        return Err(err.into());
      }
    };
    if search_chars.is_empty() {
      return Err(ExternalCommandFailure::default()
        .message(move |e: EmbedBuilder| e
          .description(&format!("Could not find any character by the name {}.", name)))
        .wrap());
    }
    let character = self.bot.xivdb.character(search_chars[0]["id"].as_u64().unwrap()).unwrap();
    if character.name.to_lowercase() != name.to_lowercase() {
      return Err(ExternalCommandFailure::default()
        .message(move |e: EmbedBuilder| e
          .description(&format!("Could not find any character by the name {}.", name)))
        .wrap());
    }
    Ok(CommandSuccess::default()
      .message(move |e: EmbedBuilder| e.description(&format!("{} ({})", character.data.race, character.data.clan))))
  }
}
