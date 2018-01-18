mod join;
mod leave;
mod play;
mod stop;

use self::join::JoinCommand;
use self::leave::LeaveCommand;
use self::play::PlayCommand;
use self::stop::StopCommand;

use bot::data::VoiceContainer;

use lalafell::commands::prelude::*;
use lalafell::error::*;

use serenity::prelude::Mutex;
use serenity::client::bridge::voice::ClientVoiceManager;

use std::collections::HashMap;

const USAGE: &str = "!music [subcommand] (args)";

#[derive(Default)]
pub struct MusicCommand;

impl MusicCommand {
  pub fn voice_manager(ctx: &Context) -> Result<Arc<Mutex<ClientVoiceManager>>> {
    match ctx.data.lock().get::<VoiceContainer>() {
      Some(vm) => Ok(Arc::clone(vm)),
      None => return Err("No reference to voice manager. This is a bug.".into())
    }
  }
}

#[derive(Debug, Deserialize)]
pub struct Params {
  subcommand: String,
  args: Option<Vec<String>>
}

impl HasParams for MusicCommand {
  type Params = Params;
}

lazy_static! {
  static ref COMMANDS: HashMap<&'static str, Box<PublicChannelCommand<'static> + Send + Sync>> = {
    let mut map: HashMap<&'static str, Box<PublicChannelCommand<'static> + Send + Sync>> = HashMap::new();
    map.insert("join", box JoinCommand);
    map.insert("leave", box LeaveCommand);
    map.insert("play", box PlayCommand);
    map.insert("stop", box StopCommand);
    map
  };
}

impl<'a> PublicChannelCommand<'a> for MusicCommand {
  fn run(&self, ctx: &Context, msg: &Message, guild: GuildId, channel: Arc<RwLock<GuildChannel>>, str_params: &[&str]) -> CommandResult<'a> {
    let params = self.params(USAGE, str_params)?;

    let sub_params = params.args.unwrap_or_default();
    let sub_param_refs: Vec<&str> = sub_params.iter().map(AsRef::as_ref).collect();

    match COMMANDS.get(&params.subcommand.to_lowercase().as_str()) {
      Some(c) => c.run(ctx, msg, guild, channel, &sub_param_refs),
      None => Err("Invalid subcommand.".into())
    }
  }
}
