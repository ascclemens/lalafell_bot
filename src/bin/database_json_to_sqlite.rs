#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_codegen;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate dotenv;

mod schema {
  infer_schema!("dotenv:LB_DATABASE_LOCATION");
}

use diesel::Connection;
use diesel::sqlite::SqliteConnection;
use diesel::prelude::*;

use std::env::{args, var};
use std::fs::File;

fn main() {
  dotenv::dotenv().ok();

  let args: Vec<String> = args().skip(1).collect();

  let old: old_database::Database = {
    let old_db_location = &args[0];
    let file = File::open(old_db_location).unwrap();
    serde_json::from_reader(file).unwrap()
  };

  let connection = SqliteConnection::establish(&var("LB_DATABASE_LOCATION").unwrap()).unwrap();

  println!("Migrating users");
  for user in old.autotags.users {
    println!("  Migrating {} on {} ({} on {})", user.user_id, user.server_id, user.character, user.server);
    let new_tag = new_database::NewTag::new(user.user_id, user.server_id, user.character_id, &user.character, &user.server, old.autotags.last_updated);
    diesel::insert(&new_tag).into(schema::tags::table).execute(&connection).unwrap();
    if user.verification.verification_string.is_some() {
      println!("    User has verification info, so migrating verification");
      let tag: new_database::Tag = schema::tags::dsl::tags
        .filter(schema::tags::dsl::user_id.eq(user.user_id.to_string()).and(schema::tags::dsl::server_id.eq(user.server_id.to_string())))
        .first(&connection)
        .unwrap();
      let new_verification = new_database::NewVerification {
        tag_id: tag.id,
        verified: user.verification.verified,
        verification_string: user.verification.verification_string
      };
      diesel::insert(&new_verification).into(schema::verifications::table).execute(&connection).unwrap();
    }
  }
  println!("Migrating timeouts");
  for timeout in old.timeouts {
    println!("  Migrating timeout for {} on {}", timeout.user_id, timeout.server_id);
    let new_timeout = new_database::NewTimeout::new(timeout.user_id, timeout.server_id, timeout.role_id, timeout.seconds as i32, timeout.start);
    diesel::insert(&new_timeout).into(schema::timeouts::table).execute(&connection).unwrap();
  }
}

mod new_database {
  use schema::*;

  #[derive(Debug, Insertable)]
  #[table_name = "verifications"]
  pub struct NewVerification {
    pub tag_id: i32,
    pub verified: bool,
    pub verification_string: Option<String>
  }

  #[derive(Debug, Queryable)]
  pub struct Tag {
    pub id: i32,
    pub user_id: String,
    pub server_id: String,
    pub character_id: String,
    pub character: String,
    pub server: String,
    pub last_updated: i64
  }

  #[derive(Debug, Insertable)]
  #[table_name = "tags"]
  pub struct NewTag {
    pub user_id: String,
    pub server_id: String,
    pub character_id: String,
    pub character: String,
    pub server: String,
    pub last_updated: i64
  }

  impl NewTag {
    pub fn new(user_id: u64, server_id: u64, character_id: u64, character: &str, server: &str, last_updated: i64) -> Self {
      NewTag {
        user_id: user_id.to_string(),
        server_id: server_id.to_string(),
        character_id: character_id.to_string(),
        character: character.to_owned(),
        server: server.to_owned(),
        last_updated: last_updated
      }
    }
  }

  #[derive(Debug, Insertable)]
  #[table_name = "timeouts"]
  pub struct NewTimeout {
    pub user_id: String,
    pub server_id: String,
    pub role_id: String,
    pub seconds: i32,
    pub start: i64
  }

  impl NewTimeout {
    pub fn new(user_id: u64, server_id: u64, role_id: u64, seconds: i32, start: i64) -> Self {
      NewTimeout {
        user_id: user_id.to_string(),
        server_id: server_id.to_string(),
        role_id: role_id.to_string(),
        seconds: seconds,
        start: start
      }
    }
  }
}

mod old_database {
  #[derive(Debug, Default, Serialize, Deserialize)]
  pub struct Database {
    #[serde(skip_serializing, skip_deserializing)]
    pub last_saved: i64,
    pub autotags: Autotags,
    #[serde(default)]
    pub timeouts: Vec<TimeoutUser>
  }

  #[derive(Debug, Default, Serialize, Deserialize)]
  pub struct Autotags {
    pub last_updated: i64,
    pub users: Vec<AutotagUser>
  }

  #[derive(Debug, Default, Serialize, Deserialize)]
  pub struct AutotagUser {
    pub user_id: u64,
    pub server_id: u64,
    pub character_id: u64,
    pub character: String,
    pub server: String,
    #[serde(default)]
    pub verification: UserVerification
  }

  #[derive(Debug, Default, Serialize, Deserialize)]
  pub struct UserVerification {
    pub verified: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_string: Option<String>
  }

  #[derive(Debug, Serialize, Deserialize)]
  pub struct TimeoutUser {
    pub server_id: u64,
    pub user_id: u64,
    pub role_id: u64,
    pub seconds: i64,
    pub start: i64
  }
}
