use bot::BotEnv;
use listeners::{Timeouts, PollTagger, AutoReplyListener, ReactionAuthorize};
use commands::*;

use lalafell::listeners::CommandListener;

use serenity::client::{EventHandler, Context};
use serenity::model::channel::Message;
use serenity::model::gateway::{Game, Ready};

use std::sync::Arc;

pub struct Handler {
  listeners: Vec<Box<EventHandler + Send + Sync>>
}

impl Handler {
  pub fn new(env: Arc<BotEnv>) -> Self {
    let listeners: Vec<Box<EventHandler + Send + Sync>> = vec![
      box command_listener(env.clone()),
      box ReactionAuthorize,
      box Timeouts,
      box PollTagger,
      box AutoReplyListener::default()
    ];
    // FIXME: ListenerManager for config listeners
    // for listener in &bot.config.listeners {
    //   let listener = ListenerManager::from_config(bot.clone(), listener).chain_err(|| format!("could not create listener {}", listener.name))?;
    //   listeners.push(listener);
    // }
    Handler { listeners }
  }
}

impl EventHandler for Handler {
  fn message(&self, ctx: Context, msg: Message) {
    for listener in &self.listeners {
      listener.message(ctx.clone(), msg.clone());
    }
  }

  fn ready(&self, ctx: Context, _: Ready) {
    ctx.shard.set_game(Some(Game::playing("with other Lalafell.")));
  }
}

macro_rules! command_listener {
  (env => $env:expr, $($($alias:expr),+ => $name:ident),+) => {{
    let mut command_listener = CommandListener::default();
    $(
      command_listener.add_command(&[$($alias),*], box $name::new($env.clone()));
    )*
    command_listener
  }}
}

// 15/17 (not 18, not porting archive yet) = 94.12% ported

fn command_listener<'a>(env: Arc<BotEnv>) -> CommandListener<'a> {
  command_listener! {
    env => env,
    "race" => RaceCommand,
    "tag" => TagCommand,
    "autotag" => AutoTagCommand,
    "viewtag" => ViewTagCommand,
    "updatetags" => UpdateTagsCommand,
    "updatetag" => UpdateTagCommand,
    "verify" => VerifyCommand,
    "referencecount" => ReferenceCountCommand,
    "poll" => PollCommand,
    "pollresults" => PollResultsCommand,
    "timeout" => TimeoutCommand,
    "untimeout" => UntimeoutCommand,
    // "archive" => ArchiveCommand,
    "viewedits" => ViewEditsCommand,
    "imagedump", "dump" => ImageDumpCommand,
    "configure", "config" => ConfigureCommand,
    "randomreaction", "reaction" => RandomReactionCommand,
    "search" => SearchCommand
  }
}
