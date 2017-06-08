extern crate hyper;
extern crate hyper_rustls;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate scraper;
extern crate chrono;
extern crate dotenv;

use hyper::Client;

use scraper::{Html, Selector};

use chrono::{UTC, TimeZone};

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::Read;
use std::env;

const NEWS_URL: &'static str = "http://na.finalfantasyxiv.com/lodestone/news/";

struct NewsScraper {
  client: Client,
  database: NewsDatabase
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct NewsDatabase {
  items: HashMap<String, NewsItem>
}

#[derive(Debug, Serialize, Deserialize)]
struct NewsItem {
  title: String,
  url: String,
  kind: NewsKind,
  time: i64,
  tag: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
enum NewsKind {
  SpecialNotice,
  News,
  Topic
}

impl ToString for NewsKind {
  fn to_string(&self) -> String {
    match *self {
      NewsKind::News => "News",
      NewsKind::Topic => "Topic",
      NewsKind::SpecialNotice => "Special notice"
    }.to_string()
  }
}

fn main() {
  dotenv::dotenv().ok();

  let webhook_url = match env::var("LB_NEWS_WEBHOOK") {
    Ok(u) => u,
    Err(_) => {
      println!("No webhook specified");
      return;
    }
  };

  let database = match NewsDatabase::load() {
    Some(db) => db,
    None => NewsDatabase::default()
  };
  let mut scraper = NewsScraper::new(database);

  let news = match scraper.download_news() {
    Some(n) => n,
    None => {
      println!("could not download news");
      return;
    }
  };

  let downloaded_news = scraper.parse_news(&news);
  for (id, item) in downloaded_news {
    if !scraper.database.items.contains_key(&id) {
      let mut embed = json!({
        "type": "rich",
        "timestamp": UTC.timestamp(item.time, 0).to_rfc3339(),
        "fields": [
          {
            "name": "Title",
            "value": item.title,
            "inline": false
          },
          {
            "name": "Link",
            "value": item.url,
            "inline": false
          },
          {
            "name": "Kind",
            "value": item.kind.to_string(),
            "inline": true
          }
        ]
      });
      if let Some(ref tag) = item.tag {
        embed["fields"].as_array_mut().unwrap().push(json!({
          "name": "Tag",
          "value": tag,
          "inline": true
        }));
      }
      let data = json!({
        "embeds": [embed]
      });
      let res = scraper.client.post(&webhook_url)
        .header(hyper::header::ContentType::json())
        .body(&data.to_string())
        .send();
      let mut data = match res {
        Ok(r) => r,
        Err(e) => {
          println!("error sending webhook: {}", e);
          continue;
        }
      };
      let mut content = String::new();
      if let Err(e) = data.read_to_string(&mut content) {
        println!("could not read webhook response: {}", e);
        continue;
      }
      if data.status.class() != hyper::status::StatusClass::Success {
        println!("discord says no to {}", item.title);
        println!("{}", content);
      } else {
        println!("webhook sent: {}", item.title);
        scraper.database.items.insert(id, item);
      }
    }
  }
  if scraper.database.save().is_none() {
    println!("could not save database");
  }
}

impl NewsDatabase {
  fn load() -> Option<Self> {
    let news_file = match File::open("./lodestone_news.json") {
      Ok(n) => n,
      Err(_) => return None
    };
    serde_json::from_reader(news_file).ok()
  }

  fn save(&self) -> Option<()> {
    let news_file = match OpenOptions::new().create(true).write(true).open("./lodestone_news.json") {
      Ok(n) => n,
      Err(_) => return None
    };
    serde_json::to_writer(news_file, self).ok()
  }
}

impl NewsScraper {
  fn new(database: NewsDatabase) -> Self {
    NewsScraper {
      client: Client::with_connector(hyper::net::HttpsConnector::new(hyper_rustls::TlsClient::new())),
      database: database
    }
  }

  fn download_news(&self) -> Option<String> {
    let mut response = match self.client.get(NEWS_URL).send() {
      Ok(r) => r,
      Err(_) => return None
    };

    let mut content = String::new();
    if response.read_to_string(&mut content).is_err() {
      return None;
    }
    Some(content)
  }

  fn parse_news(&self, news: &str) -> HashMap<String, NewsItem> {
    let html = Html::parse_document(news);
    let news_selector = Selector::parse("div.news__content.parts__space--add > ul:nth-of-type(2) > li").unwrap();
    let topics_selector = Selector::parse("div.news__content.parts__space--add > ul:nth-of-type(3) > li").unwrap();
    let title_selector = Selector::parse("p.news__list--title").unwrap();
    let time_script_selector = Selector::parse("time.news__list--time > script").unwrap();
    let mut lis: Vec<_> = html.select(&news_selector).map(|x| (NewsKind::News, x)).collect();
    lis.append(&mut html.select(&topics_selector).map(|x| (NewsKind::Topic, x)).collect());
    let mut items = HashMap::with_capacity(lis.len());
    for (kind, li) in lis {
      let child = match kind {
        NewsKind::News => li.first_child().and_then(|v| v.value().as_element()),
        NewsKind::Topic => li.select(&title_selector).next().and_then(|v| v.first_child().and_then(|x| x.value().as_element())),
        _ => {
          println!("unsupported news kind");
          continue;
        }
      };

      let child = match child {
        Some(c) => c,
        None => {
          println!("could not get news item child");
          continue
        }
      };

      let href = match child.attr("href") {
        Some(h) => h,
        None => {
          println!("invalid link in news item");
          continue;
        }
      };

      let (title, tag) = match kind {
        NewsKind::News => {
          let title = match li.select(&title_selector).next() {
            Some(t) => t,
            None => {
              println!("missing title in news item");
              continue;
            }
          };

          let tag: Option<String> = title.first_child()
            .and_then(scraper::ElementRef::wrap)
            .map(|c| c.text().collect::<String>())
            .map(|tag| tag[1..tag.len() - 1].to_string());

          let text_iter = title.text();
          let title: String = if tag.is_some() {
            text_iter.skip(1).collect()
          } else {
            text_iter.collect()
          };

          (title, tag)
        },
        NewsKind::Topic => {
          let text = li.select(&title_selector).next()
            .and_then(|v| v.first_child())
            .and_then(scraper::ElementRef::wrap)
            .map(|v| v.text().collect());
          match text {
            Some(t) => (t, None),
            None => {
              println!("invalid topic: no title");
              continue;
            }
          }
        },
        _ => unreachable!()
      };

      let time_script = match li.select(&time_script_selector).next() {
        Some(ts) => ts,
        None => {
          println!("news item missing time script");
          continue
        }
      };

      let time_script: String = time_script.text().collect();
      let time_string = match time_script.split("strftime(").nth(1).and_then(|v| v.split(',').next()) {
        Some(time) => time,
        None => {
          println!("invalid script in news item");
          continue
        }
      };
      let time: i64 = match time_string.parse() {
        Ok(t) => t,
        Err(_) => {
          println!("invalid time in time script");
          continue
        }
      };

      let id = match href.split('/').last() {
        Some(i) => i,
        None => {
          println!("invalid href in news item");
          continue;
        }
      };

      let news_item = NewsItem {
        title: title.trim().to_string(),
        url: format!("http://na.finalfantasyxiv.com{}", href),
        kind: kind,
        time: time,
        tag: tag.map(|x| x.trim().to_string())
      };
      items.insert(id.to_string(), news_item);
    }

    items
  }
}
