use serenity::client::{Context, EventHandler};
use serenity::model::guild::Guild;

/// Extra functionality for big boi guilds.
///
/// When big boi guilds are sent in the Ready event, only their online members are sent. This makes
/// the bot request offline members, as well, or else the bot will never know their join times,
/// breaking features of the bot.
pub struct GuildsExt;

impl EventHandler for GuildsExt {
  fn guild_create(&self, ctx: Context, guild: Guild, new: bool) {
    if new || !guild.is_large() {
      return;
    }

    info!("Asking for offline users of {} ({})", guild.name, guild.id);
    ctx.shard.chunk_guilds(vec![guild.id], None, None);
  }
}
