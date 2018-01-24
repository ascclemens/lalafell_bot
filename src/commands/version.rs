use lalafell::commands::prelude::*;

#[derive(BotCommand)]
pub struct VersionCommand;

impl<'a> Command<'a> for VersionCommand {
  fn run(&self, _: &Context, _: &Message, _: &[&str]) -> CommandResult<'a> {
    Ok(include_str!(concat!(env!("OUT_DIR"), "/version")).into())
  }
}
