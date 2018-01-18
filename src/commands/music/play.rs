use commands::music::MusicCommand;

use lalafell::commands::prelude::*;

use serenity::voice::{self, Handler};
use serenity::client::bridge::voice::ClientVoiceManager;
use serenity::model::id::UserId;

const USAGE: &str = "!music play [youtube url]";

#[derive(Default)]
pub struct PlayCommand;

#[derive(Debug, Deserialize)]
pub struct Params {
  url: String
}

impl HasParams for PlayCommand {
  type Params = Params;
}

impl<'a> PublicChannelCommand<'a> for PlayCommand {
  fn run(&self, ctx: &Context, msg: &Message, guild: GuildId, _: Arc<RwLock<GuildChannel>>, params: &[&str]) -> CommandResult<'a> {
    let params = self.params(USAGE, params)?;

    let youtube = match voice::ytdl(&params.url) {
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

    handler.play(youtube);

    Ok(CommandSuccess::default())
  }
}

impl PlayCommand {
  fn join_author(vm: &mut ClientVoiceManager, author: UserId, guild_id: GuildId) -> Option<&mut Handler> {
    let guild = guild_id.find()?;
    let channel_id = guild.read().voice_states.get(&author)?.channel_id.clone()?;
    vm.join(guild_id, channel_id)
  }
}
