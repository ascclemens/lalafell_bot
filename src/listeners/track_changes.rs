use bot::LalafellBot;
use listeners::ReceivesEvents;
use discord::model::Event;

use std::sync::Arc;

#[allow(dead_code)]
pub struct TrackChanges {
  bot: Arc<LalafellBot>
}

impl TrackChanges {
  pub fn new(bot: Arc<LalafellBot>) -> Self {
    TrackChanges {
      bot: bot
    }
  }

  fn handle_message_update(&self, update: MessageUpdate) {

  }
}

impl ReceivesEvents for TrackChanges {
  fn receive(&self, event: &Event) {
    match *event {
      Event::MessageUpdate {
        id: MessageId,
        channel_id: ChannelId,
        kind: Option<MessageType>,
        content: Option<String>,
        nonce: Option<String>,
        tts: Option<bool>,
        pinned: Option<bool>,
        timestamp: Option<DateTime<FixedOffset>>,
        edited_timestamp: Option<DateTime<FixedOffset>>,
        author: Option<User>,
        mention_everyone: Option<bool>,
        mentions: Option<Vec<User>>,
        mention_roles: Option<Vec<RoleId>>,
        attachments: Option<Vec<Attachment>>,
        embeds: Option<Vec<Value>>
      } => {
        let update = MessageUpdate {
          id: id,
          channel_id: channel_id,
          kind: kind,
          content: content,
          nonce: nonce,
          tts: tts,
          pinned: pinned,
          timestamp: timestamp,
          edited_timestamp: edited_timestamp,
          author: author,
          mention_everyone: mention_everyone,
          mentions: mentions,
          mention_roles: mention_roles,
          attachments: attachments,
          embeds: embeds
        };
        self.handle_message_update(update);
      },
      _ => {}
    }
  }
}

struct MessageUpdate {
  id: MessageId,
  channel_id: ChannelId,
  kind: Option<MessageType>,
  content: Option<String>,
  nonce: Option<String>,
  tts: Option<bool>,
  pinned: Option<bool>,
  timestamp: Option<DateTime<FixedOffset>>,
  edited_timestamp: Option<DateTime<FixedOffset>>,
  author: Option<User>,
  mention_everyone: Option<bool>,
  mentions: Option<Vec<User>>,
  mention_roles: Option<Vec<RoleId>>,
  attachments: Option<Vec<Attachment>>,
  embeds: Option<Vec<Value>>
}
