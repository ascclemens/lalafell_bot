/*
!poll #announcements Poll text goes here!
:one: Option one
:two: Option two

From @Bot in #announcements
Poll from @jkcclemens:
Poll text goes here!
:one: – Option one
:two: – Option two

@Bot reacts to the message with all the options

!poll #general Some other poll
Auto-generate options
No custom emoji

!poll #channel Another custom poll
:thumbsup: Yes
:thumbsdown: No
*/

use bot::LalafellBot;
use commands::*;

use discord::model::{MessageId, ReactionEmoji};

use std::sync::Arc;

const USAGE: &'static str = "!pollresults <channel> <message id>";
const VALID_EMOJI: &'static [&'static str] = &[
  "1⃣",
  "2⃣",
  "3⃣",
  "4⃣",
  "5⃣",
  "6⃣",
  "7⃣",
  "8⃣",
  "9⃣"
];

pub struct PollResultsCommand {
  bot: Arc<LalafellBot>
}

impl PollResultsCommand {
  pub fn new(bot: Arc<LalafellBot>) -> PollResultsCommand {
    PollResultsCommand {
      bot: bot
    }
  }
}

#[derive(Debug, Deserialize)]
pub struct Params {
  channel: ChannelOrId,
  message_id: u64
}

impl HasParams for PollResultsCommand {
  type Params = Params;
}

impl<'a> Command<'a> for PollResultsCommand {
  fn run(&self, _: &Message, params: &[&str]) -> CommandResult<'a> {
    let params = self.params(USAGE, params)?;
    let channel = params.channel;
    let message_id = params.message_id;
    let message = match self.bot.discord.get_message(*channel, MessageId(message_id)) {
      Ok(m) => m,
      Err(_) => return Err("Could not get that message.".into())
    };
    let mut reactions: Vec<(&String, u64)> = message.reactions.iter()
      .map(|r| match r.emoji {
        ReactionEmoji::Unicode(ref s) if VALID_EMOJI.contains(&s.as_str()) && r.me => Some((s, r.count - 1)),
        _ => None
      })
      .filter(|x| x.is_some())
      .map(|x| x.unwrap())
      .collect();
    reactions.sort_by_key(|x| !x.1);
    let votes = reactions.iter()
      .map(|&(emoji, count)| format!("{} with {} vote{}", emoji, count, if count == 1 { "" } else { "s" }))
      .collect::<Vec<_>>()
      .join("\n");
    Ok(votes.into())
  }
}
