use error::*;

use serenity::client::{Context, EventHandler};
use serenity::model::channel::Message;

pub struct PollTagger;

impl EventHandler for PollTagger {
  result_wrap! {
    fn message(&self, _ctx: Context, mut message: Message) -> Result<()> {
      let current_user = ::serenity::CACHE.read().user.clone();
      if message.embeds.len() != 1 || message.author.id != current_user.id {
        return Ok(());
      }
      let first_embed = message.embeds[0].clone();
      let footer = match first_embed.footer.map(|f| f.text) {
        Some(f) => f,
        None => return Ok(())
      };
      let title = match first_embed.title {
        Some(t) => t,
        None => return Ok(())
      };
      let description = match first_embed.description {
        Some(d) => d,
        None => return Ok(())
      };
      if footer != "Loading poll ID..." {
        return Ok(());
      }
      let id = message.id.0;
      message.edit(|c| c.embed(|e| e
        .title(title)
        .description(description)
        .footer(|f| f.text(&format!("{}", id)))))
        .chain_err(|| "could not edit poll")
    } |e| warn!("{}", e)
  }
}
