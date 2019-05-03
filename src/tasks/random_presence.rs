use crate::{
  bot::BotEnv,
  database::models::{Presence, PresenceKind},
  tasks::RunsTask,
};

use serenity::{
  client::bridge::gateway::{
    ShardClientMessage,
    ShardManager,
    ShardRunnerMessage,
  },
  gateway::InterMessage,
  model::gateway::Activity,
  prelude::Mutex,
};

use diesel::prelude::*;

use chrono::Duration;

use rand::{thread_rng, seq::SliceRandom};

use std::{
  sync::Arc,
  thread,
};

#[derive(Debug)]
pub struct RandomPresenceTask {
  next_sleep: i64,
  shard_manager: Arc<Mutex<ShardManager>>,
}

impl RandomPresenceTask {
  pub fn new(shard_manager: Arc<Mutex<ShardManager>>) -> Self {
    RandomPresenceTask {
      next_sleep: 0,
      shard_manager,
    }
  }
}

impl RunsTask for RandomPresenceTask {
  fn start(mut self, env: Arc<BotEnv>) {
    loop {
      if self.next_sleep == 0 {
        self.next_sleep = std::cmp::max(12, env.config.read().presence.change_frequency);
      }
      thread::sleep(Duration::seconds(self.next_sleep).to_std().unwrap());
      info!("Changing presence");
      let activity = match random_activity() {
        Some(g) => g,
        None => {
          info!("No presence");
          continue;
        },
      };
      let manager = self.shard_manager.lock();
      let runners = manager.runners.lock();
      for si in runners.values() {
        let message = InterMessage::Client(
          box ShardClientMessage::Runner(ShardRunnerMessage::SetActivity(Some(activity.clone())))
        );
        if let Err(e) = si.runner_tx.send(message) {
          warn!("Could not tell shard to change presence: {}", e);
        }
      }
      info!("Done changing presence");
    }
  }
}

pub fn random_activity() -> Option<Activity> {
  let presences: Vec<Presence> = crate::bot::with_connection(|c| {
    use crate::database::schema::presences::dsl;
    dsl::presences.load(c)
  }).ok()?;
  let presence = presences.choose(&mut thread_rng())?;
  let kind = PresenceKind::from_i16(presence.kind)?.as_discord();
  Some(Activity {
    kind,
    name: presence.content.clone(),

    application_id: None,
    assets: None,
    details: None,
    flags: None,
    instance: None,
    party: None,
    secrets: None,
    state: None,
    timestamps: None,
    url: None,
  })
}
