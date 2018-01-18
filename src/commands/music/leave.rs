use commands::music::MusicCommand;

use lalafell::commands::prelude::*;

#[derive(Default)]
pub struct LeaveCommand;

impl<'a> PublicChannelCommand<'a> for LeaveCommand {
  fn run(&self, ctx: &Context, _: &Message, guild: GuildId, _: Arc<RwLock<GuildChannel>>, _: &[&str]) -> CommandResult<'a> {
    let vm = MusicCommand::voice_manager(ctx)?;
    let mut manager = vm.lock();
    if manager.leave(guild).is_some() {
      Ok(CommandSuccess::default())
    } else {
      Err(ExternalCommandFailure::default().wrap())
    }
  }
}
