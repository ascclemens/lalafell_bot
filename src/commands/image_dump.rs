use crate::database::models::{ToU64, ChannelConfig};

use diesel::prelude::*;

use lalafell::commands::prelude::*;
use lalafell::error::*;

use reqwest::blocking::Client;

use url::Url;

use chrono::Duration;

use std::sync::Arc;
use std::io::Read;

const VALID_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "gifv", "mp4", "mpeg4"];

#[derive(BotCommand)]
pub struct ImageDumpCommand;

#[derive(Debug, StructOpt)]
#[structopt(about = "Tell the bot to dump images in a text file to this channel")]
pub struct Params {
  #[structopt(help = "The URL to a file containing image URLs on each line")]
  link: Url
}

impl HasParams for ImageDumpCommand {
  type Params = Params;
}

impl<'a> PublicChannelCommand<'a> for ImageDumpCommand {
  fn run(&self, ctx: &Context, _: &Message, guild: GuildId, channel: Arc<RwLock<GuildChannel>>, params: &[&str]) -> CommandResult<'a> {
    let config: Option<ChannelConfig> = crate::bot::with_connection(|c| {
      use crate::database::schema::channel_configs::dsl;
      dsl::channel_configs
        .filter(dsl::channel_id.eq(channel.read().id.to_u64()).and(dsl::server_id.eq(guild.to_u64())))
        .first(c)
        .optional()
    }).chain_err(|| "could not load configs")?;
    if !config.and_then(|c| c.image_dump_allowed).unwrap_or(false) {
      return Err("`!imagedump` is not allowed in this channel.".into());
    }

    let params = self.params_then("imagedump", params, |a| a.setting(structopt::clap::AppSettings::ArgRequiredElseHelp))?;
    let id = channel.read().id;
    let http = Arc::clone(&ctx.http);
    std::thread::spawn(move || {
      let link = params.link;
      fn get_lines(link: &Url) -> Result<Vec<String>> {
        let client = Client::new();
        let mut res = client.get(link.clone()).send().chain_err(|| "could not download")?;
        let mut content = String::new();
        res.read_to_string(&mut content).chain_err(|| "could not read download")?;
        Ok(content.lines()
          .filter(|l| {
            let url = match Url::parse(l) {
              Ok(u) => u,
              Err(_) => return false
            };
            match url.path_segments().and_then(Iterator::last).and_then(|s| s.split('.').last()) {
              Some(p) if VALID_EXTENSIONS.contains(&p.to_lowercase().as_ref()) => true,
              _ => false
            }
          })
          .map(ToString::to_string)
          .collect())
      }
      let lines = match get_lines(&link) {
        Ok(l) => l,
        Err(_) => {
          id.send_message(http, |c| c.embed(|e| e.description("Could not download/parse that link."))).ok();
          return;
        }
      };
      for chunk in lines.chunks(5) {
        id.send_message(&http, |c| c.content(chunk.join("\n"))).ok();
        std::thread::sleep(Duration::seconds(1).to_std().unwrap());
      }
    });
    Ok(CommandSuccess::default())
  }
}
