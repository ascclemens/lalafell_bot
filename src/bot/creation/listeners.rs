use bot::BotEnv;
use commands::*;
use listeners::*;

use lalafell::listeners::CommandListener;

use serenity::prelude::RwLock;
use serenity::model::prelude::*;
use serenity::client::{EventHandler, Context};
use serenity::client::bridge::gateway::event::ShardStageUpdateEvent;

use serde_json::Value;

use std::sync::Arc;
use std::collections::HashMap;

pub struct Handler {
  listeners: Vec<Box<EventHandler + Send + Sync>>
}

impl Handler {
  pub fn new(env: &Arc<BotEnv>) -> Self {
    let listeners: Vec<Box<EventHandler + Send + Sync>> = vec![
      box command_listener(env),
      box ReactionAuthorize,
      box Timeouts,
      box PollTagger,
      box AutoReplyListener::default(),
      box PartyFinder::default(),
      box Music,
      box Log
    ];
    Handler { listeners }
  }
}

macro_rules! handler {
  ($name:ident, $($param:ident: $kind:ty),+) => {
    fn $name(&self, $($param: $kind),+) {
      for listener in &self.listeners {
        listener.$name($($param.clone()),+);
      }
    }
  }
}

impl EventHandler for Handler {
  handler!(cached, param1: Context, param2: Vec < GuildId >);
  handler!(category_create, param1: Context, param2: Arc < RwLock < ChannelCategory > >);
  handler!(category_delete, param1: Context, param2: Arc < RwLock < ChannelCategory > >);
  handler!(channel_create, param1: Context, param2: Arc < RwLock < GuildChannel > >);
  handler!(channel_delete, param1: Context, param2: Arc < RwLock < GuildChannel > >);
  handler!(channel_pins_update, param1: Context, param2: ChannelPinsUpdateEvent);
  handler!(channel_recipient_addition, param1: Context, param2: ChannelId, param3: User);
  handler!(channel_recipient_removal, param1: Context, param2: ChannelId, param3: User);
  handler!(channel_update, param1: Context, param2: Option < Channel >, param3: Channel);
  handler!(guild_ban_addition, param1: Context, param2: GuildId, param3: User);
  handler!(guild_ban_removal, param1: Context, param2: GuildId, param3: User);
  handler!(guild_create, param1: Context, param2: Guild, param3: bool);
  handler!(guild_delete, param1: Context, param2: PartialGuild, param3: Option < Arc < RwLock < Guild > > >);
  handler!(guild_emojis_update, param1: Context, param2: GuildId, param3: HashMap < EmojiId , Emoji >);
  handler!(guild_integrations_update, param1: Context, param2: GuildId);
  handler!(guild_member_addition, param1: Context, param2: GuildId, param3: Member);
  handler!(guild_member_removal, param1: Context, param2: GuildId, param3: User, param4: Option < Member >);
  handler!(guild_member_update, param1: Context, param2: Option < Member >, param3: Member);
  handler!(guild_members_chunk, param1: Context, param2: GuildId, param3: HashMap < UserId , Member >);
  handler!(guild_role_create, param1: Context, param2: GuildId, param3: Role);
  handler!(guild_role_delete, param1: Context, param2: GuildId, param3: RoleId, param4: Option < Role >);
  handler!(guild_role_update, param1: Context, param2: GuildId, param3: Option < Role >, param4: Role);
  handler!(guild_unavailable, param1: Context, param2: GuildId);
  handler!(guild_update, param1: Context, param2: Option < Arc < RwLock < Guild > > >, param3: PartialGuild);
  handler!(message_delete_bulk, param1: Context, param2: ChannelId, param3: Vec < MessageId >);
  handler!(message_delete, param1: Context, param2: ChannelId, param3: MessageId);
  handler!(message_update, param1: Context, param2: MessageUpdateEvent);
  handler!(message, param1: Context, param2: Message);
  handler!(presence_replace, param1: Context, param2: Vec < Presence >);
  handler!(presence_update, param1: Context, param2: PresenceUpdateEvent);
  handler!(private_channel_create, param1: Context, param2: Arc < RwLock < PrivateChannel > >);
  handler!(reaction_add, param1: Context, param2: Reaction);
  handler!(reaction_remove_all, param1: Context, param2: ChannelId, param3: MessageId);
  handler!(reaction_remove, param1: Context, param2: Reaction);
  handler!(resume, param1: Context, param2: ResumedEvent);
  handler!(shard_stage_update, param1: Context, param2: ShardStageUpdateEvent);
  handler!(typing_start, param1: Context, param2: TypingStartEvent);
  handler!(unknown, param1: Context, param2: String, param3: Value);
  handler!(user_update, param1: Context, param2: CurrentUser, param3: CurrentUser);
  handler!(voice_server_update, param1: Context, param2: VoiceServerUpdateEvent);
  handler!(voice_state_update, param1: Context, param2: Option < GuildId >, param3: VoiceState);
  handler!(webhook_update, param1: Context, param2: GuildId, param3: ChannelId);

  fn ready(&self, ctx: Context, ready: Ready) {
    // TODO: move this to a random presence listener
    if let Some(g) = ::tasks::random_presence::random_game() {
      ctx.shard.set_game(Some(g));
    }
    for listener in &self.listeners {
      listener.ready(ctx.clone(), ready.clone());
    }
  }
}

macro_rules! command_listener {
  (env => $env:expr, $($($alias:expr),+ => $name:ident),+) => {{
    let mut command_listener = CommandListener::new("!");
    $(
      command_listener.add_command(&[$($alias),*], box $name::new(Arc::clone($env)));
    )*
    command_listener
  }}
}

fn command_listener<'a>(env: &Arc<BotEnv>) -> CommandListener<'a> {
  command_listener! {
    env => env,
    "archive" => ArchiveCommand,
    "autotag" => AutoTagCommand,
    "blob" => BlobCommand,
    "bot" => ActualBotCommand,
    "configure", "config" => ConfigureCommand,
    "imagedump", "dump" => ImageDumpCommand,
    "mention" => MentionCommand,
    "music" => MusicCommand,
    "ping" => PingCommand,
    "poll" => PollCommand,
    "pollresults" => PollResultsCommand,
    "race" => RaceCommand,
    "randomreaction", "reaction" => RandomReactionCommand,
    "referencecount" => ReferenceCountCommand,
    "reload", "reloadconfig" => ReloadConfigCommand,
    "search" => SearchCommand,
    "tag" => TagCommand,
    "timeout" => TimeoutCommand,
    "untimeout" => UntimeoutCommand,
    "updatetag" => UpdateTagCommand,
    "updatetags" => UpdateTagsCommand,
    "verify" => VerifyCommand,
    "version" => VersionCommand,
    "viewtag" => ViewTagCommand
  }
}
