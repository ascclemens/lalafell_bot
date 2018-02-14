mod join;
mod leave;
mod pause;
mod play;
mod stop;

use self::join::JoinCommand;
use self::leave::LeaveCommand;
use self::pause::PauseCommand;
use self::play::PlayCommand;
use self::stop::StopCommand;

use bot::data::VoiceContainer;

use lalafell::commands::prelude::*;
use lalafell::error::*;

use serenity::prelude::Mutex;
use serenity::client::bridge::voice::ClientVoiceManager;

#[derive(BotCommand)]
pub struct MusicCommand;

impl MusicCommand {
  pub fn voice_manager(ctx: &Context) -> Result<Arc<Mutex<ClientVoiceManager>>> {
    match ctx.data.lock().get::<VoiceContainer>() {
      Some(vm) => Ok(Arc::clone(vm)),
      None => Err("No reference to voice manager. This is a bug.".into())
    }
  }
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Control the bot's music capabilities.")]
pub enum Params {
  #[structopt(name = "join", about = "Tell the bot to join a channel")]
  #[structopt(raw(template = "::lalafell::commands::TEMPLATE"))]
  Join(join::Params),
  #[structopt(name = "leave", about = "Tell the bot to leave the channel it's in")]
  Leave,
  #[structopt(name = "pause", alias = "resume", about = "Tell the bot to pause or resume the currently playing song")]
  Pause,
  #[structopt(name = "play", about = "Tell the bot to play a song")]
  #[structopt(raw(template = "::lalafell::commands::TEMPLATE"))]
  Play(play::Params),
  #[structopt(name = "stop", about = "Tell the bot to stop playing")]
  Stop
}

impl HasParams for MusicCommand {
  type Params = Params;
}

struct Commands {
  join: JoinCommand,
  leave: LeaveCommand,
  play: PlayCommand,
  pause: PauseCommand,
  stop: StopCommand
}

lazy_static! {
  static ref COMMANDS: Commands = Commands {
    join: JoinCommand,
    leave: LeaveCommand,
    play: PlayCommand,
    pause: PauseCommand,
    stop: StopCommand
  };
}

impl<'a> PublicChannelCommand<'a> for MusicCommand {
  fn run(&self, ctx: &Context, msg: &Message, guild: GuildId, channel: Arc<RwLock<GuildChannel>>, str_params: &[&str]) -> CommandResult<'a> {
    let params = self.params("music", str_params)?;

    match params {
      Params::Join(p) => COMMANDS.join.run(ctx, msg, guild, channel, p),
      Params::Leave => COMMANDS.leave.run(ctx, msg, guild, channel),
      Params::Play(p) => COMMANDS.play.run(ctx, msg, guild, channel, p),
      Params::Pause => COMMANDS.pause.run(ctx, msg, guild, channel),
      Params::Stop => COMMANDS.stop.run(ctx, msg, guild, channel)
    }
  }
}
