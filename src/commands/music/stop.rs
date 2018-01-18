use commands::music::MusicCommand;

use lalafell::commands::prelude::*;

#[derive(Default)]
pub struct StopCommand;

impl<'a> PublicChannelCommand<'a> for StopCommand {
  fn run(&self, ctx: &Context, _: &Message, guild: GuildId, _: Arc<RwLock<GuildChannel>>, _: &[&str]) -> CommandResult<'a> {
    let vm = MusicCommand::voice_manager(ctx)?;
    let mut manager = vm.lock();
    let handler = match manager.get_mut(guild) {
      Some(h) => h,
      None => return Err("I'm not in any voice channels.".into())
    };
    handler.stop();
    Ok(CommandSuccess::default())
  }
}
