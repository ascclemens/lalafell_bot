use commands::music::MusicCommand;

use lalafell::commands::prelude::*;

#[derive(BotCommand)]
pub struct LeaveCommand;

impl<'a> LeaveCommand {
  pub fn run(&self, ctx: &Context, _: &Message, guild: GuildId, _: Arc<RwLock<GuildChannel>>) -> CommandResult<'a> {
    let vm = MusicCommand::voice_manager(ctx)?;
    let mut manager = vm.lock();
    if manager.leave(guild).is_some() {
      Ok(CommandSuccess::default())
    } else {
      Err(ExternalCommandFailure::default().wrap())
    }
  }
}
