use lalafell::commands::prelude::*;

use serenity::model::id::UserId;
use serenity::model::channel::Channel;
use serenity::builder::CreateEmbed;

use lalafell::error::*;

use std::boxed::FnBox;

const USAGE: &str = "!poll <poll text>\n<option>\n<option>...";

#[derive(Default)]
pub struct PollCommand;

impl PollCommand {
  fn nick_or_name(&self, guild: GuildId, user: UserId) -> Option<String> {
    match guild.member(user) {
      Ok(m) => Some(m.display_name().to_string()),
      Err(_) => None
    }
  }
}

impl<'a> Command<'a> for PollCommand {
  fn run(&self, _: &Context, msg: &Message, _: &[&str]) -> CommandResult<'a> {
    let lines: Vec<&str> = msg.content.split('\n').collect();
    let params: Vec<&str> = lines[0].split_whitespace().skip(1).collect();
    let options = &lines[1..];
    if params.is_empty() || options.len() < 2 {
      return Err(ExternalCommandFailure::default()
        .message(|e: CreateEmbed| e
          .title("Not enough parameters.")
          .description(USAGE))
        .wrap());
    }
    if options.len() > 9 {
      return Err("No more than nine poll options can be specified.".into());
    }
    let message = params.join(" ");
    let channel = match msg.channel_id.get() {
      Ok(Channel::Guild(c)) => c,
      _ => return Err("This command must be used in a guild channel.".into())
    };
    msg.delete().chain_err(|| "could not delete original message")?;
    let name = self.nick_or_name(channel.read().guild_id, msg.author.id).unwrap_or_else(|| "someone".into());
    let poll = Poll::new(name, &message, options);
    channel.read().send_message(|c| c
      .embed(poll.create_embed())
      .reactions((0..poll.options.len()).map(|i| format!("{}⃣", i + 1))))
      .chain_err(|| "could not send embed")?;
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
      author,
      text: text.to_string(),
      options: options.iter().map(|x| x.to_string()).collect()
    }
  }

  fn create_embed(&self) -> Box<FnBox(CreateEmbed) -> CreateEmbed> {
    let name = self.author.clone();
    let options = self.options.iter()
      .enumerate()
      .map(|(i, x)| format!("{}⃣ – {}", i + 1, x))
      .collect::<Vec<_>>()
      .join("\n");
    let desc = format!("{}\n{}", self.text, options);
    box move |e: CreateEmbed| {
      e
        .title(&format!("Poll by {}", name))
        .description(&desc)
        .footer(|f| f.text("Loading poll ID..."))
    }
  }
}
