use lalafell::error::*;

use reqwest::Client;

use scraper::{Html, Selector};

use std::io::Read;

pub struct Lodestone {
  client: Client
}

impl Lodestone {
  pub fn new() -> Lodestone {
    Lodestone {
      client: Client::new()
    }
  }

  pub fn character_profile(&self, id: u64) -> Result<String> {
    let mut res = self.client
      .get(&format!("https://na.finalfantasyxiv.com/lodestone/character/{}/", id))
      .send()
      .chain_err(|| "could not download from lodestone")?;
    let mut content = String::new();
    res.read_to_string(&mut content).chain_err(|| "could not read data from lodestone")?;
    let html = Html::parse_document(&content);
    let selector = Selector::parse("div.character__selfintroduction").unwrap();
    let profile = match html.select(&selector).next() {
      Some(p) => p,
      None => bail!("could not find character__selfintroduction")
    };
    Ok(profile.text().collect::<Vec<_>>().join(" ").trim().to_string())
  }
}
