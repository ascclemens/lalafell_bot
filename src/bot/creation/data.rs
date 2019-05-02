use crate::bot::LalafellBot;

use typemap::Key;

use serenity::client::bridge::gateway::ShardManager;
use serenity::client::bridge::voice::ClientVoiceManager;
use serenity::model::id::ChannelId;
use serenity::prelude::Mutex;
use serenity::voice::LockedAudio;

use std::collections::HashMap;
use std::sync::Arc;

pub fn data(bot: &LalafellBot) {
  let mut data = bot.discord.data.write();
  data.insert::<ShardManagerContainer>(Arc::clone(&bot.discord.shard_manager));
  data.insert::<VoiceContainer>(Arc::clone(&bot.discord.voice_manager));
  data.insert::<AudioContainer>(Default::default());
}

pub struct ShardManagerContainer;

impl Key for ShardManagerContainer {
  type Value = Arc<Mutex<ShardManager>>;
}

pub struct VoiceContainer;

impl Key for VoiceContainer {
  type Value = Arc<Mutex<ClientVoiceManager>>;
}

pub struct AudioContainer;

impl Key for AudioContainer {
  type Value = HashMap<ChannelId, Vec<LockedAudio>>;
}
