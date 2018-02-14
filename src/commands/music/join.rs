use commands::music::MusicCommand;

use lalafell::commands::ChannelOrId;
use lalafell::commands::prelude::*;

#[derive(BotCommand)]
pub struct JoinCommand;

#[derive(Debug, StructOpt)]
#[structopt(about = "Tell the bot to join a voice channel")]
pub struct Params {
  // FIXME: Take String and check against voice channels
  #[structopt(help = "The voice channel to join")]
  channel: ChannelOrId
}

impl<'a> JoinCommand {
  pub fn run(&self, ctx: &Context, _: &Message, guild: GuildId, _: Arc<RwLock<GuildChannel>>, params: Params) -> CommandResult<'a> {
    let vm = MusicCommand::voice_manager(ctx)?;
    let mut manager = vm.lock();
    if manager.join(guild, *params.channel).is_some() {
      Ok(CommandSuccess::default())
    } else {
      Err(ExternalCommandFailure::default().wrap())
    }
  }
}
