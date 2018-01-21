use bot::LalafellBot;

use typemap::Key;

use serenity::prelude::Mutex;
use serenity::client::bridge::gateway::ShardManager;
use serenity::client::bridge::voice::ClientVoiceManager;

use std::sync::Arc;

pub fn data(bot: &LalafellBot) {
  let mut data = bot.discord.data.lock();
  data.insert::<ShardManagerContainer>(Arc::clone(&bot.discord.shard_manager));
  data.insert::<VoiceContainer>(Arc::clone(&bot.discord.voice_manager));
}

pub struct ShardManagerContainer;

impl Key for ShardManagerContainer {
  type Value = Arc<Mutex<ShardManager>>;
}

pub struct VoiceContainer;

impl Key for VoiceContainer {
  type Value = Arc<Mutex<ClientVoiceManager>>;
}
