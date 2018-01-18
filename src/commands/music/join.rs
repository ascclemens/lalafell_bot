use commands::music::MusicCommand;

use lalafell::commands::ChannelOrId;
use lalafell::commands::prelude::*;

const USAGE: &str = "!music join [channel]";

#[derive(Default)]
pub struct JoinCommand;

#[derive(Debug, Deserialize)]
pub struct Params {
  // FIXME: Take String and check against voice channels
  channel: ChannelOrId
}

impl HasParams for JoinCommand {
  type Params = Params;
}

impl<'a> PublicChannelCommand<'a> for JoinCommand {
  fn run(&self, ctx: &Context, _: &Message, guild: GuildId, _: Arc<RwLock<GuildChannel>>, params: &[&str]) -> CommandResult<'a> {
    let params = self.params(USAGE, params)?;

    let vm = MusicCommand::voice_manager(ctx)?;
    let mut manager = vm.lock();
    if manager.join(guild, *params.channel).is_some() {
      Ok(CommandSuccess::default())
    } else {
      Err(ExternalCommandFailure::default().wrap())
    }
  }
}
