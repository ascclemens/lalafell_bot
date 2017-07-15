use bot::LalafellBot;
use lodestone::Lodestone;
use database::models::{Tag, Verification};

use lalafell::error;
use lalafell::error::*;
use lalafell::bot::Bot;
use lalafell::commands::prelude::*;

use diesel::prelude::*;

use discord::builders::EmbedBuilder;
use discord::model::{Message, LiveServer, PublicChannel};

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

impl HasBot for VerifyCommand {
  fn bot(&self) -> &Bot {
    self.bot.as_ref()
  }
}

impl<'a> PublicChannelCommand<'a> for VerifyCommand {
  fn run(&self, message: &Message, server: &LiveServer, _: &PublicChannel, _: &[&str]) -> CommandResult<'a> {
    let server_id = server.id;
    let user: Option<Tag> = ::bot::CONNECTION.with(|c| {
      use database::schema::tags::dsl;
      dsl::tags
        .filter(dsl::user_id.eq(message.author.id.0.to_string()).and(dsl::server_id.eq(server_id.0.to_string())))
        .first(c)
        .optional()
        .chain_err(|| "could not load tags")
    })?;
    let user = match user {
      Some(u) => u,
      None => return Err(ExternalCommandFailure::default()
        .message(|e: EmbedBuilder| e
          .title("Not tagged.")
          .description("Please tag yourself with an account before verifying it."))
        .wrap())
    };
    let verification: Verification = ::bot::CONNECTION.with(|c| {
      Verification::belonging_to(&user)
        .first(c)
        .optional()
        .chain_err(|| "could not load verifications")
    })?.unwrap_or_default();
    if verification.verified {
      return Err("You are already verified.".into());
    }
    let verification_string = match verification.verification_string {
      Some(ref v) => v,
      None => {
        let mut new_verification = verification.into_new(user.id);
        let verification_string = new_verification.create_verification_string().clone();
        ::bot::CONNECTION.with(move |c| {
          use database::schema::verifications;
          ::diesel::insert(&new_verification).into(verifications::table)
            .execute(c)
            .chain_err(|| "could not insert verification")
        })?;
        let chan = self.bot.discord.create_private_channel(message.author.id).chain_err(|| "could not create private channel")?;
        self.bot.discord.send_embed(chan.id, "", |e| e
          .title("Verification instructions")
          .description(&format!("Edit your Lodestone profile to contain `{}`.\nRerun the `!verify` command afterward.", verification_string))
          .url("http://na.finalfantasyxiv.com/lodestone/my/setting/profile/")).ok();
        return Ok(CommandSuccess::default());
      }
    };
    let profile = Lodestone::new().character_profile(*user.character_id)?;
    if profile.contains(verification_string) {
      let state_option = self.bot.state.read().unwrap();
      let state = state_option.as_ref().unwrap();
      let server = match state.servers().iter().find(|x| x.id == server_id) {
        Some(s) => s,
        None => {
          let err: error::Error = "could not find server for channel".into();
          return Err(err.into());
        }
      };

      ::bot::CONNECTION.with(|c| {
        use database::schema::verifications::dsl;
        ::diesel::update(&verification)
          .set(dsl::verified.eq(true))
          .execute(c)
          .chain_err(|| "could not update verification")
      })?;
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
