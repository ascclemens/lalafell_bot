use database::models::{ToU64, NewEphemeralMessage};

use chrono::{Utc, Duration, DateTime};

use diesel;
use diesel::prelude::*;

use lalafell::error::*;
use lalafell::commands::prelude::*;
use lalafell::commands::ChannelOrId;

use serenity::model::id::{ChannelId, MessageId};

use std::sync::Arc;

#[derive(BotCommand)]
pub struct EphemeralMessageCommand;

#[derive(Debug, StructOpt)]
#[structopt(about = "Set a message to expire after a given time")]
pub struct Params {
  #[structopt(
    short = "c",
    long = "channel",
    help = "The channel the message is in"
  )]
  channel: ChannelOrId,
  #[structopt(
    short = "m",
    long = "message",
    help = "The ID of the message to delete"
  )]
  message: u64,
  #[structopt(help = "The date to delete the message")]
  time: DateTime<Utc>
}

impl HasParams for EphemeralMessageCommand {
  type Params = Params;
}

impl<'a> PublicChannelCommand<'a> for EphemeralMessageCommand {
  fn run(&self, _: &Context, _: &Message, guild_id: GuildId, _: Arc<RwLock<GuildChannel>>, params: &[&str]) -> CommandResult<'a> {
    let params = self.params_then("ephemeralmessage", params, |a| a.setting(::structopt::clap::AppSettings::ArgRequiredElseHelp))?;
    if params.time < Utc::now() {
      return Err("Cannot create an ephemeral message with an expiration date in the past.".into());
    }
    let nem = NewEphemeralMessage::new(guild_id.0, params.channel.0, params.message, params.time.timestamp());
    ::bot::with_connection(|c| {
      use database::schema::ephemeral_messages::dsl;

      diesel::insert_into(dsl::ephemeral_messages).values(&nem).execute(c)
    }).chain_err(|| "could not insert new ephemeral message")?;

    // spawn a task if the message needs to be deleted sooner than 30 minutes from now
    if params.time <= Utc::now() + Duration::minutes(30) {
      let dur = params.time.signed_duration_since(Utc::now());
      spawn_task(*params.channel, MessageId(params.message), dur);
    }

    Ok(CommandSuccess::default())
  }
}

fn spawn_task(channel: ChannelId, message: MessageId, after: Duration) {
  ::std::thread::spawn(move || {
    ::std::thread::sleep(after.to_std().unwrap());
    if let Err(e) = channel.delete_message(message) {
      warn!("could not delete ephemeral message {} in {}: {}", message, channel, e);
      return;
    }
    let res = ::bot::with_connection(|c| {
      use database::schema::ephemeral_messages::dsl;

      diesel::delete(dsl::ephemeral_messages
        .filter(dsl::channel_id.eq(channel.to_u64()).and(dsl::message_id.eq(message.to_u64()))))
        .execute(c)
    });
    if let Err(e) = res {
      warn!("could not delete ephemeral message from database ({} in {}): {}", message, channel, e);
    }
  });
}
