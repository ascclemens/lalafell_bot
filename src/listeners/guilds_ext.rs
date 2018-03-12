use serenity::client::{Context, EventHandler};
use serenity::model::id::GuildId;
use serenity::model::gateway::Ready;
use serenity::model::guild::GuildStatus;

/// Extra functionality for big boi guilds.
///
/// When big boi guilds are sent in the Ready event, only their online members are sent. This makes
/// the bot request offline members, as well, or else the bot will never know their join times,
/// breaking features of the bot.
pub struct GuildsExt;

impl EventHandler for GuildsExt {
  fn ready(&self, ctx: Context, rdy: Ready) {
    let ids: Vec<GuildId> = rdy.guilds
      .into_iter()
      .filter_map(|g| match g {
        GuildStatus::OnlinePartialGuild(partial) => Some(partial),
        _ => None
      })
      .map(|partial| partial.id)
      .collect();
    info!("Asking for more information about {} guild{}",
      ids.len(),
      if ids.len() == 1 { "" } else { "s" }
    );
    ctx.shard.chunk_guilds(ids, None, None);
  }
}
