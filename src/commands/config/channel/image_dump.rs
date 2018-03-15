use database::models::{ToU64, ChannelConfig, NewChannelConfig};

use serenity::prelude::Mentionable;
use serenity::builder::CreateEmbed;
use serenity::model::id::{GuildId, ChannelId, UserId};

use diesel;
use diesel::prelude::*;

use lalafell::error::*;
use lalafell::commands::prelude::*;

#[derive(Default, BotCommand)]
pub struct ImageDumpCommand;

#[derive(Debug, StructOpt)]
pub struct Params {
  #[structopt(help = "Whether to enable image dumps for this channel or not")]
  enabled: Option<String>
}

impl<'a> ImageDumpCommand {
  pub fn run(&self, author: UserId, guild: GuildId, channel: ChannelId, params: Params) -> CommandResult<'a> {
    let member = guild.member(author).chain_err(|| "could not get member")?;
    if !member.permissions().chain_err(|| "could not get permissions")?.manage_messages() {
      return Err(ExternalCommandFailure::default()
        .message(|e: CreateEmbed| e
          .title("Not enough permissions.")
          .description("You don't have enough permissions to use this command."))
        .wrap());
    }
    let config: Option<ChannelConfig> = ::bot::with_connection(|c| {
      use database::schema::channel_configs::dsl;
      dsl::channel_configs
        .filter(dsl::server_id.eq(guild.to_u64()).and(dsl::channel_id.eq(channel.to_u64())))
        .first(c)
        .optional()
    }).chain_err(|| "could not load channel configs")?;
    let enabled = match params.enabled {
      Some(e) => match e.to_lowercase().as_str() {
        "enabled" | "enable" | "on" | "true" | "yes" => true,
        "disabled" | "disable" | "off" | "false" | "no" => false,
        _ => return Err("Unknown enabled state provided".into())
      },
      None => {
        let status = match config.and_then(|c| c.image_dump_allowed) {
          Some(true) => "enabled",
          Some(false) => "disabled",
          None => "unset (disabled)"
        };
        return Ok(format!("`!imagedump` status in {}: {}", channel.mention(), status).into());
      }
    };
    match config {
      Some(mut conf) => {
        conf.image_dump_allowed = Some(enabled);
        ::bot::with_connection(|c| conf.save_changes::<ChannelConfig>(c)).chain_err(|| "could not update config")?;
      },
      None => {
        ::bot::with_connection(|c| {
          let new = NewChannelConfig {
            server_id: guild.into(),
            channel_id: channel.into(),
            image_dump_allowed: Some(enabled)
          };
          diesel::insert_into(::database::schema::channel_configs::table)
            .values(&new)
            .execute(c)
        }).chain_err(|| "could not add config")?;
      }
    }
    Ok(format!("Set `!imagedump` status in {} to {}.", channel.mention(), if enabled { "enabled" } else { "disabled" }).into())
  }
}
