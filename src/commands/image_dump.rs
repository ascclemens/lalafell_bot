use bot::BotEnv;
use database::models::ChannelConfig;

use diesel::prelude::*;

use lalafell::commands::prelude::*;
use lalafell::error::*;

use make_hyper_great_again::Client;
use hyper_rustls::HttpsConnector;

use url::Url;
use url_serde;

use chrono::Duration;

use std::sync::Arc;
use std::io::Read;

const USAGE: &str = "!imagedump <url>";
const VALID_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "gifv", "mp4", "mpeg4"];

pub struct ImageDumpCommand;

impl ImageDumpCommand {
  pub fn new(_: Arc<BotEnv>) -> Self {
    ImageDumpCommand
  }
}

#[derive(Debug, Deserialize)]
pub struct Params {
  #[serde(with = "url_serde")]
  link: Url
}

impl HasParams for ImageDumpCommand {
  type Params = Params;
}

impl<'a> PublicChannelCommand<'a> for ImageDumpCommand {
  fn run(&self, _: &Context, _: &Message, guild: GuildId, channel: Arc<RwLock<GuildChannel>>, params: &[&str]) -> CommandResult<'a> {
    let config: Option<ChannelConfig> = ::bot::CONNECTION.with(|c| {
      use database::schema::channel_configs::dsl;
      dsl::channel_configs
        .filter(dsl::channel_id.eq(channel.read().id.0.to_string()).and(dsl::server_id.eq(guild.0.to_string())))
        .first(c)
        .optional()
        .chain_err(|| "could not load configs")
    })?;
    if !config.and_then(|c| c.image_dump_allowed).unwrap_or(false) {
      return Err("`!imagedump` is not allowed in this channel.".into());
    }

    let params = self.params(USAGE, params)?;
    let id = channel.read().id;
    ::std::thread::spawn(move || {
      let link = params.link;
      fn get_lines(link: &Url) -> Result<Vec<String>> {
        let client = Client::create_connector(|c| HttpsConnector::new(4, &c.handle())).chain_err(|| "could not create client")?;
        let mut res = client.get(link).send().chain_err(|| "could not download")?;
        let mut content = String::new();
        res.read_to_string(&mut content).chain_err(|| "could not read download")?;
        Ok(content.lines()
          .filter(|l| {
            let url = match Url::parse(l) {
              Ok(u) => u,
              Err(_) => return false
            };
            match url.path_segments().and_then(|s| s.last()).and_then(|s| s.split('.').last()) {
              Some(p) if VALID_EXTENSIONS.contains(&p.to_lowercase().as_ref()) => true,
              _ => false
            }
          })
          .map(|x| x.to_string())
          .collect())
      }
      let lines = match get_lines(&link) {
        Ok(l) => l,
        Err(_) => {
          id.send_message(|c| c.embed(|e| e.description("Could not download/parse that link."))).ok();
          return;
        }
      };
      for chunk in lines.chunks(5) {
        id.send_message(|c| c.content(chunk.join("\n"))).ok();
        ::std::thread::sleep(Duration::seconds(1).to_std().unwrap());
      }
    });
    Ok(CommandSuccess::default())
  }
}
