use bot::BotEnv;
use commands::BotCommand as BotCommandTrait;
use database::models::{Presence, NewPresence, PresenceKind};

use lalafell::error::*;
use lalafell::commands::prelude::*;

use serenity::model::gateway::{Game, GameType};

use diesel::prelude::*;

use itertools::Itertools;

const USAGE: &str = "!bot [subcommand] (args)";

pub struct BotCommand {
  env: Arc<BotEnv>
}

impl BotCommandTrait for BotCommand {
  fn new(env: Arc<BotEnv>) -> Self {
    BotCommand { env }
  }
}

impl HasParams for BotCommand {
  type Params = Params;
}

#[derive(Debug, Deserialize)]
pub struct Params {
  subcommand: String,
  args: Option<Vec<String>>
}

impl<'a> Command<'a> for BotCommand {
  fn run(&self, ctx: &Context, message: &Message, params: &[&str]) -> CommandResult<'a> {
    if !self.env.config.read().bot.administrators.contains(&message.author.id.0) {
      return Err(ExternalCommandFailure::default()
        .message(|e: CreateEmbed| e
          .title("Not enough permissions.")
          .description("You don't have enough permissions to use this command."))
        .wrap());
    }
    let params = self.params(USAGE, params)?;
    let args = params.args.unwrap_or_default();
    match params.subcommand.as_ref() {
      "presence" | "presences" => self.presence(ctx, &args),
      _ => Err("Invalid subcommand.".into())
    }
  }
}

impl BotCommand {
  fn presence<'a>(&self, ctx: &Context, args: &[String]) -> CommandResult<'a> {
    if args.len() < 1 {
      return self.list_all_presences();
    }
    let subcommand = &args[0];
    let args = &args[1..];
    match subcommand.as_str() {
      "add" | "create" => self.add_presence(args),
      "remove" | "delete" => self.remove_presence(args),
      "change" | "set" => self.change_presence(ctx, args),
      "random" => self.random_presence(ctx),
      _ => Err("Invalid subcommand".into())
    }
  }

  fn list_all_presences<'a>(&self) -> CommandResult<'a> {
    let presences: Vec<Presence> = ::bot::CONNECTION.with(|c| {
      use database::schema::presences::dsl;
      dsl::presences.load(c).chain_err(|| "could not load presences")
    })?;
    let strings = presences.iter()
      .map(|p| format!("{}. {} {}", p.id, PresenceKind::from_i16(p.kind).map(|x| x.to_string()).unwrap_or_else(|| "<invalid type>".to_string()), p.content))
      .join("\n");
    Ok(strings.into())
  }

  fn change_presence<'a>(&self, ctx: &Context, args: &[String]) -> CommandResult<'a> {
    if args.is_empty() {
      return Err("!bot presence change [playing/listening] [content]".into());
    }
    let game_type = match args[0].as_str() {
      "playing" => GameType::Playing,
      "listening" => GameType::Listening,
      _ => return Err("Invalid presence type.".into())
    };
    let game = Game {
      kind: game_type,
      name: args[1..].join(" "),
      url: None
    };
    ctx.set_game(game);
    Ok(CommandSuccess::default())
  }

  fn random_presence<'a>(&self, ctx: &Context) -> CommandResult<'a> {
    match ::tasks::random_presence::random_game() {
      Some(g) => ctx.set_game(g),
      None => return Err("No presences.".into())
    }
    Ok(CommandSuccess::default())
  }

  fn remove_presence<'a>(&self, args: &[String]) -> CommandResult<'a> {
    if args.is_empty() {
      return Err("!bot presences remove [id]".into());
    }
    let id: i32 = args[0].parse().map_err(|_| into!(CommandFailure, "Invalid ID."))?;
    let affected = ::bot::CONNECTION.with(|c| {
      use database::schema::presences::dsl;
      ::diesel::delete(dsl::presences.filter(dsl::id.eq(id)))
        .execute(c)
        .chain_err(|| "could not delete presence")
    })?;
    if affected > 0 {
      Ok(CommandSuccess::default())
    } else {
      Err("No presence with that ID were found.".into())
    }
  }

  fn add_presence<'a>(&self, args: &[String]) -> CommandResult<'a> {
    if args.len() < 2 {
      return Err("!bot presences add [playing/listening] [content]".into());
    }
    let kind = match args[0].as_str() {
      "playing" => PresenceKind::Playing,
      "listening" => PresenceKind::Listening,
      _ => return Err("Invalid presence kind.".into())
    };
    let content = args[1..].join(" ");
    ::bot::CONNECTION.with(|c| {
      use database::schema::presences::dsl;
      ::diesel::insert_into(dsl::presences)
        .values(&NewPresence::new(kind as i16, &content))
        .execute(c)
        .chain_err(|| "could not add new presence")
    })?;
    Ok(CommandSuccess::default())
  }
}
