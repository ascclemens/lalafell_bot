use bot::LalafellBot;

use typemap::Key;

use serenity::prelude::Mutex;
use serenity::client::bridge::gateway::ShardManager;

use std::sync::Arc;

pub fn data(bot: &LalafellBot) {
  let mut data = bot.discord.data.lock();
  data.insert::<ShardManagerContainer>(Arc::clone(&bot.discord.shard_manager));
}

pub struct ShardManagerContainer;

impl Key for ShardManagerContainer {
  type Value = Arc<Mutex<ShardManager>>;
}
