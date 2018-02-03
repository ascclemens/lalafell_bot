use bot::data::AudioContainer;
use commands::music::MusicCommand;

use lalafell::commands::prelude::*;
use lalafell::error::*;

#[derive(BotCommand)]
pub struct PauseCommand;

impl<'a> PublicChannelCommand<'a> for PauseCommand {
  fn run(&self, ctx: &Context, _: &Message, guild: GuildId, _: Arc<RwLock<GuildChannel>>, _: &[&str]) -> CommandResult<'a> {
    let vm = MusicCommand::voice_manager(ctx)?;
    let manager = vm.lock();

    let handler = match manager.get(guild) {
      Some(h) => h,
      None => return Err("I'm not in any voice channel.".into())
    };

    let mut data = ctx.data.lock();
    let container = data.get_mut::<AudioContainer>().chain_err(|| "could not get audio container")?;
    let audios = container.entry(handler.channel_id.chain_err(|| "no channel id")?).or_insert_with(Default::default);
    for audio in audios {
      let mut audio = audio.lock();
      if audio.finished {
        continue;
      }
      if audio.playing {
        audio.pause();
      } else {
        audio.play();
      }
    }

    Ok(CommandSuccess::default())
  }
}
