use bot::data::AudioContainer;
use commands::music::MusicCommand;

use lalafell::commands::prelude::*;
use lalafell::error::*;

use serenity::voice::{self, Handler};
use serenity::client::bridge::voice::ClientVoiceManager;
use serenity::model::id::UserId;

use url::Url;

#[derive(BotCommand)]
pub struct PlayCommand;

#[derive(Debug, StructOpt)]
#[structopt(about = "Tell the bot to play a YouTube video's audio")]
pub struct Params {
  #[structopt(help = "The YouTube URL to play")]
  url: Url
}

impl<'a> PlayCommand {
  #[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
  pub fn run(&self, ctx: &Context, msg: &Message, guild: GuildId, _: Arc<RwLock<GuildChannel>>, params: Params) -> CommandResult<'a> {
    let youtube = match voice::ytdl(&params.url.to_string()) {
      Ok(y) => y,
      Err(e) => return Err(format!("Could not open that YouTube URL: {}", e).into())
    };

    let vm = MusicCommand::voice_manager(ctx)?;
    let mut manager = vm.lock();

    // FIXME: this is stupid
    let has_handler = manager.get(guild).is_some();
    let handler = if has_handler {
      manager.get_mut(guild)
    } else {
      PlayCommand::join_author(&mut manager, msg.author.id, guild)
    };

    let handler = handler.ok_or_else(|| into!(CommandFailure, "I'm not in any voice channel and neither are you."))?;

    let audio = handler.play_returning(youtube);
    {
      let mut data = ctx.data.lock();
      let container = data.get_mut::<AudioContainer>().chain_err(|| "could not get audio container")?;
      let audios = container.entry(handler.channel_id.chain_err(|| "no channel id")?).or_insert_with(Default::default);
      audios.retain(|a| !a.lock().finished);
      audios.push(audio);
    }

    Ok(CommandSuccess::default())
  }
}

impl PlayCommand {
  fn join_author(vm: &mut ClientVoiceManager, author: UserId, guild_id: GuildId) -> Option<&mut Handler> {
    let guild = guild_id.find()?;
    let channel_id = guild.read().voice_states.get(&author)?.channel_id?;
    vm.join(guild_id, channel_id)
  }
}
