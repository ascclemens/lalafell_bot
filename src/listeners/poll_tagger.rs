use bot::LalafellBot;
use listeners::ReceivesEvents;
use discord::model::{Event, Message};
use discord::builders::EmbedBuilder;

use std::sync::Arc;

pub struct PollTagger {
  bot: Arc<LalafellBot>
}

impl PollTagger {
  pub fn new(bot: Arc<LalafellBot>) -> Self {
    PollTagger { bot: bot }
  }

  fn tag_poll(&self, message: &Message) {
    let current_user = match self.bot.discord.get_current_user() {
      Ok(c) => c,
      Err(e) => {
        warn!("couldn't get current user: {}", e);
        return;
      }
    };
    if message.embeds.len() != 1 && message.author.id != current_user.id {
      return;
    }
    let footer = match message.embeds[0].get("footer").and_then(|f| f.get("text")).and_then(|t| t.as_str()) {
      Some(f) => f,
      None => return
    };
    let title = match message.embeds[0].get("title").and_then(|t| t.as_str()) {
      Some(t) => t,
      None => return
    };
    let description = match message.embeds[0].get("description").and_then(|d| d.as_str()) {
      Some(d) => d,
      None => return
    };
    if footer != "Loading poll ID..." {
      return;
    }
    self.bot.discord.edit_embed(message.channel_id, message.id, |e: EmbedBuilder| e
      .title(title)
      .description(description)
      .footer(|f| f.text(&format!("{}", message.id.0))))
      .ok();
  }
}

impl ReceivesEvents for PollTagger {
  fn receive(&self, event: &Event) {
    match *event {
      Event::MessageCreate(ref msg) => self.tag_poll(msg),
      _ => {}
    }
  }
}
