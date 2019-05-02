use bot::data::ShardManagerContainer;

use lalafell::commands::prelude::*;

use serenity::client::bridge::gateway::ShardId;

use chrono::Duration;

#[derive(BotCommand)]
pub struct PingCommand;

impl<'a> Command<'a> for PingCommand {
  fn run(&self, ctx: &Context, _: &Message, _: &[&str]) -> CommandResult<'a> {
    let sm = match ctx.data.read().get::<ShardManagerContainer>() {
      Some(sm) => Arc::clone(sm),
      None => return Err("No reference to shard manager. This is a bug.".into())
    };
    let manager = sm.lock();
    let runners = manager.runners.lock();
    match runners.get(&ShardId(ctx.shard_id)) {
      Some(info) => Ok(format!("{} ms", info.latency
        .map(Duration::from_std)
        .and_then(Result::ok)
        .map(|x| x.num_milliseconds().to_string())
        .unwrap_or_else(|| "<unknown>".into())).into()),
      None => Err("Shard not found. This is a bug.".into())
    }
  }
}
