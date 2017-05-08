use LalafellBot;
use commands::*;
use lodestone::Lodestone;

use discord::builders::EmbedBuilder;
use discord::model::Channel;

use xivdb::error::*;

use std::sync::Arc;

pub struct VerifyCommand {
  bot: Arc<LalafellBot>
}

impl VerifyCommand {
  pub fn new(bot: Arc<LalafellBot>) -> VerifyCommand {
    VerifyCommand {
      bot: bot
    }
  }
}

impl<'a> Command<'a> for VerifyCommand {
  fn run(&self, message: &Message, _: &[&str]) -> CommandResult<'a> {
    let channel = self.bot.discord.get_channel(message.channel_id).chain_err(|| "could not get channel for message")?;
    let server_id = match channel {
      Channel::Public(c) => c.server_id,
      _ => {
        let err: error::Error = "channel was not public".into();
        return Err(err.into());
      }
    };
    let mut database = self.bot.database.lock().unwrap();
    let user = database.autotags.users.iter_mut().find(|u| u.user_id == message.author.id.0 && u.server_id == server_id.0);
    let mut user = match user {
      Some(u) => u,
      None => return Err(ExternalCommandFailure::default()
        .message(|e: EmbedBuilder| e
          .title("Not tagged.")
          .description("Please tag yourself with an account before verifying it."))
        .wrap())
    };
    if user.verification.verified {
      return Err(ExternalCommandFailure::default()
        .message(|e: EmbedBuilder| e.description("You are already verified."))
        .wrap());
    }
    let verification_string = match user.verification.verification_string {
      Some(ref v) => v,
      None => {
        let verification_string = user.verification.create_verification_string();
        let chan = self.bot.discord.create_private_channel(message.author.id).chain_err(|| "could not create private channel")?;
        self.bot.discord.send_embed(chan.id, "", |e| e
          .title("Verification instructions")
          .description(&format!("Edit your Lodestone profile to contain `{}`.\nRerun the `!verify` command afterward.", verification_string))
          .url("http://na.finalfantasyxiv.com/lodestone/my/setting/profile/")).ok();
        return Ok(CommandSuccess::default());
      }
    };
    let profile = Lodestone::new().character_profile(user.character_id)?;
    if profile.contains(verification_string) {
      let mut state_option = self.bot.state.lock().unwrap();
      let state = state_option.as_mut().unwrap();
      let server = match state.servers().iter().find(|x| x.id == server_id) {
        Some(s) => s,
        None => {
          let err: error::Error = "could not find server for channel".into();
          return Err(err.into());
        }
      };

      user.verification.verified = true;
      if let Some(r) = server.roles.iter().find(|x| x.name.to_lowercase() == "verified") {
        let mut member = self.bot.discord.get_member(server_id, message.author.id).chain_err(|| "could not get member for tagging")?;

        if !member.roles.contains(&r.id) {
          member.roles.push(r.id);
          self.bot.discord.edit_member_roles(server_id, message.author.id, &member.roles).chain_err(|| "could not add roles")?;
        }
      }
      let char_name = user.character.clone();
      let serv_name = user.server.clone();
      Ok(CommandSuccess::default()
        .message(move |e: EmbedBuilder| e
          .title("Verified!")
          .description(&format!("You have successfully verified yourself as {} on {}.", char_name, serv_name))))
    } else {
      Err(ExternalCommandFailure::default()
        .wrap())
    }
  }
}
