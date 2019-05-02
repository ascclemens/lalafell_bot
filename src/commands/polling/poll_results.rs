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

use crate::commands::*;

use lalafell::commands::prelude::*;

use serenity::model::channel::ReactionType;

const VALID_EMOJI: &[&str] = &[
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

#[derive(BotCommand)]
pub struct PollResultsCommand;

#[derive(Debug, StructOpt)]
#[structopt(about = "View the results of a poll")]
pub struct Params {
  #[structopt(help = "The channel the poll was posted in")]
  channel: ChannelOrId,
  #[structopt(help = "The message ID of the poll")]
  message_id: u64
}

impl HasParams for PollResultsCommand {
  type Params = Params;
}

impl<'a> Command<'a> for PollResultsCommand {
  fn run(&self, ctx: &Context, _: &Message, params: &[&str]) -> CommandResult<'a> {
    let params = self.params_then("pollresults", params, |a| a.setting(::structopt::clap::AppSettings::ArgRequiredElseHelp))?;
    let channel = params.channel;
    let message_id = params.message_id;
    let message = match channel.message(&ctx, message_id) {
      Ok(m) => m,
      Err(_) => return Err("Could not get that message.".into())
    };
    let mut reactions: Vec<(&String, u64)> = message.reactions.iter()
      .filter_map(|r| match r.reaction_type {
        ReactionType::Unicode(ref s) if VALID_EMOJI.contains(&s.as_str()) && r.me => Some((s, r.count - 1)),
        _ => None
      })
      .collect();
    reactions.sort_by_key(|x| !x.1);
    let votes = reactions.iter()
      .map(|&(emoji, count)| format!("{} with {} vote{}", emoji, count, if count == 1 { "" } else { "s" }))
      .collect::<Vec<_>>()
      .join("\n");
    Ok(votes.into())
  }
}
