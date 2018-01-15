use bot::BotEnv;
use tasks::RunsTask;
use database::models::{Presence, PresenceKind};

use serenity::prelude::Mutex;
use serenity::model::gateway::Game;
use serenity::client::bridge::gateway::{ShardClientMessage, ShardRunnerMessage};
use serenity::client::bridge::gateway::ShardManager;

use diesel::prelude::*;

use chrono::Duration;

use rand::{Rng, thread_rng};

use std::sync::Arc;
use std::thread;

#[derive(Debug)]
pub struct RandomPresenceTask {
  next_sleep: i64,
  shard_manager: Arc<Mutex<ShardManager>>
}

impl RandomPresenceTask {
  pub fn new(shard_manager: Arc<Mutex<ShardManager>>) -> Self {
    RandomPresenceTask {
      next_sleep: 0,
      shard_manager
    }
  }
}

impl RunsTask for RandomPresenceTask {
  fn start(mut self, env: Arc<BotEnv>) {
    loop {
      if self.next_sleep == 0 {
        self.next_sleep = ::std::cmp::max(12, env.config.read().presence.change_frequency);
      }
      thread::sleep(Duration::seconds(self.next_sleep).to_std().unwrap());
      info!("Changing presence");
      let game = match random_game() {
        Some(g) => g,
        None => {
          info!("No presence");
          continue;
        }
      };
      let manager = self.shard_manager.lock();
      let runners = manager.runners.lock();
      for si in runners.values() {
        let message = ShardClientMessage::Runner(ShardRunnerMessage::SetGame(Some(game.clone())));
        if let Err(e) = si.runner_tx.send(message) {
          warn!("Could not tell shard to change presence: {}", e);
        }
      }
      info!("Done changing presence");
    }
  }
}

pub fn random_game() -> Option<Game> {
  let presences: Vec<Presence> = ::bot::CONNECTION.with(|c| {
    use database::schema::presences::dsl;
    dsl::presences.load(c).ok()
  })?;
  let presence = thread_rng().choose(&presences)?;
  let game_type = PresenceKind::from_i16(presence.kind)?.as_discord();
  Some(Game {
    kind: game_type,
    name: presence.content.clone(),
    url: None
  })
}
