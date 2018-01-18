use bot::data::VoiceContainer;

use serenity::client::{Context, EventHandler};
use serenity::model::id::GuildId;
use serenity::model::voice::VoiceState;

use std::sync::Arc;

#[allow(dead_code)]
pub struct Music;

impl EventHandler for Music {
  fn voice_state_update(&self, ctx: Context, guild: Option<GuildId>, state: VoiceState) {
    let guild = some_or!(guild.and_then(|x| x.find()), return);
    let bot_id = ::serenity::CACHE.read().user.id;
    // if the bot is leaving, stop playing
    if state.user_id == bot_id && state.channel_id.is_none() {
      let vm = Arc::clone(some_or!(ctx.data.lock().get::<VoiceContainer>(), return));
      let mut vm = vm.lock();
      some_or!(vm.get_mut(guild.read().id), return).stop();
      return;
    }
    // recount the channel status, and if the bot is alone, leave
    let bot_channel = some_or!(guild.read().voice_states.iter().find(|x| x.0 == &bot_id), return).1.channel_id.clone();
    if guild.read().voice_states.values().filter(|x| x.channel_id == bot_channel).count() == 1 {
      let vm = Arc::clone(some_or!(ctx.data.lock().get::<VoiceContainer>(), return));
      let mut vm = vm.lock();
      vm.leave(guild.read().id);
    }
  }
}
