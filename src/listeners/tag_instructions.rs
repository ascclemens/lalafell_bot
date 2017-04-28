use LalafellBot;
use listeners::ReceivesEvents;
use discord::model::{Event, Channel, ChannelId};

use std::sync::Arc;

pub struct TagInstructions {
  bot: Arc<LalafellBot>
}

impl TagInstructions {
  pub fn new(bot: Arc<LalafellBot>) -> TagInstructions {
    TagInstructions {
      bot: bot
    }
  }
}

impl ReceivesEvents for TagInstructions {
  fn receive(&self, event: &Event) {
    let destination = ChannelId(307359970096185345);
    let event_data = match *event {
      Event::ServerMemberAdd(_, ref member) => Some(member.clone()),
      Event::MessageCreate(ref m) => {
        let chan = match self.bot.discord.get_channel(m.channel_id) {
          Ok(Channel::Public(c)) => c,
          _ => return
        };
        if chan.id != destination {
          return;
        }
        if m.content.to_lowercase().starts_with("!autotag ") {
          return;
        }
        self.bot.discord.get_member(chan.server_id, m.author.id).ok()
      },
      _ => return
    };
    let member = match event_data {
      Some(e) => e,
      None => return
    };

    if !member.roles.is_empty() {
      return;
    }

    let send = format!("Welcome to the lala world linkshell, {}!\nIn order to talk with the rest of us, please tag yourself using the command below.\n`!autotag server character`\nFor example, you might send `!autotag Adamantoise Duvicauroix Priorfaix`",
      member.user.mention());
    self.bot.discord.send_embed(destination, "", |e| e.description(&send)).ok();
  }
}
