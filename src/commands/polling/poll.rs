use bot::LalafellBot;
use commands::*;

use discord::model::{ServerId, UserId, ReactionEmoji, Channel};
use discord::builders::EmbedBuilder;

use error::*;

use std::sync::Arc;

const USAGE: &'static str = "!poll <poll text>\n<option>\n<option>...";

pub struct PollCommand {
  bot: Arc<LalafellBot>
}

impl PollCommand {
  pub fn new(bot: Arc<LalafellBot>) -> PollCommand {
    PollCommand {
      bot: bot
    }
  }
}

impl PollCommand {
  fn nick_or_name(&self, server: ServerId, user: UserId) -> Option<String> {
    match self.bot.discord.get_member(server, user) {
      Ok(m) => Some(m.nick.unwrap_or(m.user.name)),
      Err(_) => None
    }
  }
}

impl<'a> Command<'a> for PollCommand {
  fn run(&self, msg: &Message, _: &[&str]) -> CommandResult<'a> {
    let lines: Vec<&str> = msg.content.split('\n').collect();
    let params: Vec<&str> = lines[0].split_whitespace().skip(1).collect();
    let options = &lines[1..];
    if params.is_empty() || options.len() < 2 {
      return Err(ExternalCommandFailure::default()
        .message(|e: EmbedBuilder| e
          .title("Not enough parameters.")
          .description(USAGE))
        .wrap());
    }
    if options.len() > 9 {
      return Err(ExternalCommandFailure::default()
        .message(|e: EmbedBuilder| e
          .description("No more than nine poll options can be specified."))
        .wrap());
    }
    let message = params.join(" ");
    let channel = match self.bot.discord.get_channel(msg.channel_id) {
      Ok(Channel::Public(c)) => c,
      _ => return Err(ExternalCommandFailure::default()
        .message(|e: EmbedBuilder| e
          .description("This command must be used in a public channel."))
        .wrap())
    };
    self.bot.discord.delete_message(msg.channel_id, msg.id).chain_err(|| "could not delete original message")?;
    let name = self.nick_or_name(channel.server_id, msg.author.id).unwrap_or_else(|| "someone".into());
    let poll = Poll::new(name, &message, options);
    let embed = self.bot.discord.send_embed(channel.id, "", poll.create_embed()).chain_err(|| "could not send embed")?;
    for i in 0..poll.options.len() {
      self.bot.discord.add_reaction(embed.channel_id, embed.id, ReactionEmoji::Unicode(format!("{}⃣", i + 1))).chain_err(|| "could not add reaction")?;
    }
    Ok(CommandSuccess::default())
  }
}

struct Poll {
  author: String,
  text: String,
  options: Vec<String>
}

impl Poll {
  fn new(author: String, text: &str, options: &[&str]) -> Poll {
    Poll {
      author: author,
      text: text.to_string(),
      options: options.iter().map(|x| x.to_string()).collect()
    }
  }

  fn create_embed(&self) -> Box<FnBox(EmbedBuilder) -> EmbedBuilder> {
    let name = self.author.clone();
    let options = self.options.iter()
      .enumerate()
      .map(|(i, x)| format!("{}⃣ – {}", i + 1, x))
      .collect::<Vec<_>>()
      .join("\n");
    let desc = format!("{}\n{}", self.text, options);
    box move |e: EmbedBuilder| {
      e
        .title(&format!("Poll by {}", name))
        .description(&desc)
    }
  }
}
