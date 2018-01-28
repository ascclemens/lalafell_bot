use database::models::{ToU64, PartyFinderConfig};

use diesel::prelude::*;

use serenity::prelude::{RwLock, Mentionable};
use serenity::client::{Context, EventHandler};
use serenity::model::channel::{Reaction, Message, GuildChannel};
use serenity::model::id::{GuildId, ChannelId, UserId, MessageId};

use error::*;

use std::collections::HashMap;

const DATA_CENTERS: &[&str] = &[
  "aether",
  "chaos",
  "elemental",
  "gaia",
  "mana",
  "primal"
];

type StatusRegistry = HashMap<UserId, (PfStatus, Option<MessageId>)>;

#[derive(Default)]
pub struct PartyFinder {
  statuses: RwLock<StatusRegistry>
}

impl EventHandler for PartyFinder {
  result_wrap! {
    fn reaction_add(&self, _ctx: Context, reaction: Reaction) -> Result<()> {
      let channel = reaction.channel()
        .chain_err(|| "could not get channel")?
        .guild()
        .chain_err(|| "could not get guild channel")?;
      let guild_id = channel.read().guild_id;
      let config = match PartyFinder::config(guild_id, channel.read().id) {
        Some(c) => c,
        None => return Ok(())
      };
      if reaction.message_id != *config.message_id {
        return Ok(());
      }
      if reaction.emoji != ::util::parse_emoji(&config.emoji) {
        return Ok(());
      }
      if let Err(e) = reaction.delete() {
        warn!("could not delete reaction: {}", e);
      }
      let mut member = guild_id.member(reaction.user_id).chain_err(|| "could not get member")?;
      if let Err(e) = member.add_role(*config.role_id) {
        bail!("could not add role: {}", e);
      }
      let mut writer = self.statuses.write();
      let entry = writer.entry(reaction.user_id).or_insert((PfStatus::Started, None));
      match entry.0 {
        PfStatus::Started => {},
        _ => return Ok(())
      }
      let res = channel.read()
        .send_message(|c| c.embed(|e| e
          .description(&format!("{}\n\nLet's make a PF ad for you!\n\nWhat data center is this PF for?", reaction.user_id.mention()))));
      match res {
        Ok(m) => { entry.1 = Some(m.id); },
        Err(e) => bail!("could not send message: {}", e)
      }
      Ok(())
    } |e| warn!("{}", e)
  }

  result_wrap! {
    fn message(&self, _ctx: Context, message: Message) -> Result<()> {
      if message.author.id == ::serenity::CACHE.read().user.id {
        return Ok(());
      }
      let channel = message.channel()
        .chain_err(|| "could not get channel")?
        .guild()
        .chain_err(|| "could not get guild channel")?;
      let config = match PartyFinder::config(channel.read().guild_id, channel.read().id) {
        Some(c) => c,
        None => return Ok(())
      };
      if let Err(e) = message.delete() {
        warn!("could not delete message: {}", e);
      }
      if message.content.to_lowercase().trim() == "cancel" {
        let removed = self.statuses.write().remove(&message.author.id);
        if let Some((_, Some(r))) = removed {
          message.channel_id.delete_message(r).ok();
        }
        return Ok(());
      }
      let (status, message_id) = match self.statuses.read().get(&message.author.id) {
        Some(&(ref a, Some(b))) => (a.clone(), b),
        _ => return Ok(())
      };
      status.process(&config, message_id, &message, channel.as_ref(), &self.statuses)
    } |e| warn!("{}", e)
  }
}

impl PartyFinder {
  fn config(guild: GuildId, channel: ChannelId) -> Option<PartyFinderConfig> {
    ::bot::CONNECTION.with(|c| {
      use database::schema::party_finder_configs::dsl;
      dsl::party_finder_configs
        .filter(dsl::server_id.eq(guild.to_u64()).and(dsl::channel_id.eq(channel.to_u64())))
        .first(c)
        .ok()
    })
  }
}

#[derive(Clone)]
pub enum PfStatus {
  Started,
  DataCenter { data_center: String },
  Duty { data_center: String, duty: String }
}

impl PfStatus {
  fn process(&self, config: &PartyFinderConfig, message_id: MessageId, message: &Message, channel: &RwLock<GuildChannel>, statuses: &RwLock<StatusRegistry>) -> Result<()> {
    match *self {
      PfStatus::Started => self.process_started(config, message_id, message, channel, statuses),
      PfStatus::DataCenter { ref data_center } => self.process_data_center(data_center, config, message_id, message, channel, statuses),
      PfStatus::Duty { ref data_center, ref duty } => self.process_duty(data_center, duty, config, message_id, message, channel, statuses)
    }
  }

  fn process_started(&self, _: &PartyFinderConfig, message_id: MessageId, message: &Message, channel: &RwLock<GuildChannel>, statuses: &RwLock<StatusRegistry>) -> Result<()> {
    let data_center = message.content.trim().to_lowercase();
    if !DATA_CENTERS.contains(&data_center.as_str()) {
      return channel.read().edit_message(message_id, |m| m
        .embed(|e| e
          .description(&format!(
            "{}\n\n`{}` isn't a valid data center. Let's try again.\n\nWhat data center is this PF for?",
            message.author.mention(),
            data_center
          ))))
        .map(|_| ())
        .chain_err(|| "could not edit message");
    }

    channel.read().edit_message(message_id, |m| m
      .embed(|e| e
        .description(&format!(
          "{}\n\nOkay, we're making the PF for the {} data center.\n\nWhat duty is the PF for?",
          message.author.mention(),
          data_center
        ))))
      .chain_err(|| "could not edit message")?;
    statuses.write().get_mut(&message.author.id).unwrap().0 = PfStatus::DataCenter { data_center };
    Ok(())
  }

  fn process_data_center(&self, data_center: &str, _: &PartyFinderConfig, message_id: MessageId, message: &Message, channel: &RwLock<GuildChannel>, statuses: &RwLock<StatusRegistry>) -> Result<()> {
    let duty = message.content.trim().to_string();
    channel.read().edit_message(message_id, |m| m
      .embed(|e| e
        .description(&format!(
          "{}\n\nOkay, we're making the PF for `{}` on the {} data center.\n\nWhat message would you like to include?",
          message.author.mention(),
          duty,
          data_center
        ))))
      .chain_err(|| "could not edit message")?;
    statuses.write().get_mut(&message.author.id).unwrap().0 = PfStatus::Duty {
      data_center: data_center.to_string(),
      duty
    };
    Ok(())
  }

  #[cfg_attr(feature = "cargo-clippy", allow(too_many_arguments))]
  fn process_duty(&self, data_center: &str, duty: &str, config: &PartyFinderConfig, message_id: MessageId, message: &Message, channel: &RwLock<GuildChannel>, statuses: &RwLock<StatusRegistry>) -> Result<()> {
    let guild_id = channel.read().guild_id;
    let mut member = guild_id.member(&message.author).chain_err(|| "could not get member")?;
    member.remove_role(*config.role_id).ok();
    statuses.write().remove(&message.author.id);
    let msg = message.content.trim().to_string();
    let chan_name = guild_id
      .find()
      .and_then(|g| g.read().channels.values()
        .find(|c| c.read().name == *data_center)
        .map(|c| c.read().id.mention().to_string()))
      .unwrap_or_else(|| data_center.to_string());
    channel.read().edit_message(message_id, |m| m
      .embed(|e| e
        .description(&format!("**PF listing by {}**", message.author.mention()))
        .fields(vec![
          ("Data center", &chan_name, true),
          ("Duty", &duty.to_string(), true),
          ("Message", &msg, false)
        ])))
      .chain_err(|| "could not edit message")?;
    Ok(())
  }
}
