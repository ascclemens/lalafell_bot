use crate::database::models::{ToU64, LogChannel};

use chrono::{Utc, Duration};

use diesel::prelude::*;

use serenity::{
  client::{Context, EventHandler},
  model::{
    channel::Message,
    event::MessageUpdateEvent,
    guild::Member,
    id::{GuildId, ChannelId, UserId, MessageId},
    user::User,
  },
  prelude::{Mutex, Mentionable},
};

use std::{
  collections::HashMap,
  sync::atomic::{Ordering, AtomicUsize},
};

macro_rules! update_message {
  ($message:expr, $update:expr, $($field:ident),+ $(,)?) => {{
    $(
      if let Some(f) = $update.$field {
        $message.$field = f;
      }
    )+
  }}
}

#[derive(Default)]
pub struct Log {
  messages: Mutex<HashMap<UserId, HashMap<GuildId, Vec<Message>>>>,
  count: AtomicUsize,
}

impl Log {
  fn get_log_channel<G: Into<GuildId>>(&self, guild: G) -> Option<ChannelId> {
    let guild = guild.into().to_u64();
    let log_channel: Option<LogChannel> = crate::bot::with_connection(|c| {
      use crate::database::schema::log_channels::dsl;
      dsl::log_channels.filter(dsl::server_id.eq(guild)).first(c)
    }).ok();
    log_channel.map(|x| ChannelId(*x.channel_id))
  }

  fn prune_messages(&self) {
    let now = Utc::now();
    let one_day = Duration::days(1);
    self.messages
      .lock()
      .values_mut()
      .flat_map(HashMap::values_mut)
      .for_each(|x| x.retain(|m| m.timestamp.signed_duration_since(now) < one_day));
  }
}

impl EventHandler for Log {
  fn guild_member_removal(&self, ctx: Context, guild: GuildId, user: User, member: Option<Member>) {
    let channel_id = some_or!(self.get_log_channel(guild), return);
    let mention = member.as_ref().map(Mentionable::mention).unwrap_or_else(|| user.mention());
    channel_id.send_message(&ctx, |m| m.embed(|mut embed| {
      embed = embed
        .author(|a| a
          .name(&user.tag())
          .icon_url(&user.face()));
      if let Some(member) = member {
        embed = embed.field("Display name", member.display_name(), true);
      }
      embed = embed
        .field("Mention", mention, true)
        .field("Action", "Leave server", true)
        .timestamp(&Utc::now())
        .footer(|f| f.text(user.id));
      embed
    })).ok();
  }

  fn guild_member_addition(&self, ctx: Context, guild: GuildId, member: Member) {
    let channel_id = some_or!(self.get_log_channel(guild), return);
    channel_id.send_message(&ctx, |m| m.embed(|mut embed| {
      embed = embed
        .author(|a| a
          .name(&member.user.read().tag())
          .icon_url(&member.user.read().face()));
      embed = embed
        .field("Mention", member.mention(), true)
        .field("Action", "Join server", true)
        .timestamp(&Utc::now())
        .footer(|f| f.text(member.user.read().id));
      embed
    })).ok();
  }

  fn message_update(&self, ctx: Context, _: Option<Message>, _: Option<Message>, update: MessageUpdateEvent) {
    let author = match update.author {
      Some(ref a) => a,
      None => return,
    };
    let new_content = match update.content {
      Some(ref c) => c.to_owned(),
      None => return,
    };

    let channel = match update.channel_id.to_channel(&ctx) {
      Ok(c) => c,
      Err(e) => {
        warn!("could not download channel {} for message history: {}", update.channel_id, e);
        return;
      },
    };

    let guild_channel = match channel.guild() {
      Some(g) => g,
      None => return,
    };
    let reader = guild_channel.read();

    let channel_id = some_or!(self.get_log_channel(reader.guild_id), return);

    let guild = match reader.guild_id.to_guild_cached(&ctx) {
      Some(g) => g,
      None => return,
    };
    let guild_reader = guild.read();

    let member = match guild_reader.members.get(&author.id).cloned().or_else(|| guild_reader.member(&ctx, author.id).ok()) {
      Some(m) => m,
      None => return,
    };

    let mut messages = self.messages.lock();
    let message = messages
      .get_mut(&author.id)
      .and_then(|x| x.get_mut(&reader.guild_id))
      .and_then(|x| x.iter_mut().find(|m| m.id == update.id));
    let message = match message {
      Some(m) => m,
      None => return,
    };

    let original_content = message.content.clone();
    let channel_mention = update.channel_id.mention();
    let message_id = update.id;

    update_message!(
      message,
      update,
      kind, content, tts, pinned, timestamp, author, mention_everyone, mentions, mention_roles,
      attachments,
    );
    message.edited_timestamp = update.edited_timestamp;

    channel_id.send_message(&ctx, |m| m.embed(|mut embed| {
      embed = embed
        .author(|a| a
          .name(&member.user.read().tag())
          .icon_url(&member.user.read().face()));
      embed = embed
        .field("Mention", member.mention(), true)
        .field("Action", "Edited message", true)
        .field("Channel", channel_mention, true)
        .field("Original message", original_content, false)
        .field("New message", &new_content, false)
        .timestamp(&Utc::now())
        .footer(|f| f.text(message_id));
      embed
    })).ok();
  }

  fn message_delete(&self, ctx: Context, channel_id: ChannelId, message_id: MessageId) {
    let channel = match channel_id.to_channel(&ctx) {
      Ok(c) => c,
      Err(e) => {
        warn!("could not download channel {} for message history: {}", channel_id, e);
        return;
      },
    };

    let guild_channel = match channel.guild() {
      Some(g) => g,
      None => return,
    };
    let reader = guild_channel.read();

    let log_channel = some_or!(self.get_log_channel(reader.guild_id), return);

    let guild = match reader.guild_id.to_guild_cached(&ctx) {
      Some(g) => g,
      None => return,
    };

    let guild_reader = guild.read();

    let mut messages = self.messages.lock();
    let message = messages
      .values_mut()
      .filter_map(|x| x.get_mut(&reader.guild_id))
      .next()
      .and_then(|x| x.iter_mut().find(|m| m.id == message_id));
    let message = match message {
      Some(m) => m.clone(),
      None => return,
    };

    let deletee = some_or!(guild_reader.members.get(&message.author.id).cloned().or_else(|| guild_reader.member(&ctx, message.author.id).ok()), return);

    let original_content = message.content;
    let channel_mention = channel_id.mention();

    log_channel.send_message(&ctx, |m| m.embed(|mut embed| {
      embed = embed
        .author(|a| a
          .name(&deletee.user.read().tag())
          .icon_url(&deletee.user.read().face()));
      embed = embed
        .field("Mention", deletee.mention(), true)
        .field("Action", "Had message deleted", true)
        .field("Channel", channel_mention, true)
        .field("Content", original_content, false)
        .timestamp(&Utc::now())
        .footer(|f| f.text(message_id));
      embed
    })).ok();
  }

  fn message(&self, ctx: Context, message: Message) {
    if self.count.load(Ordering::SeqCst) == 100 {
      self.count.store(0, Ordering::SeqCst);

      self.prune_messages();
    }

    let channel = match message.channel_id.to_channel(&ctx) {
      Ok(c) => c,
      Err(e) => {
        warn!("could not download channel {} for message history: {}", message.channel_id, e);
        return;
      },
    };

    let guild_channel = match channel.guild() {
      Some(g) => g,
      None => return,
    };

    let reader = guild_channel.read();

    self.messages
      .lock()
      .entry(message.author.id)
      .or_default()
      .entry(reader.guild_id)
      .or_default()
      .push(message);

    self.count.fetch_add(1, Ordering::SeqCst);
  }
}
