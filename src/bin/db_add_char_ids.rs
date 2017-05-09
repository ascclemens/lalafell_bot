extern crate xivdb;
extern crate serde_json;

use xivdb::XivDb;

use serde_json::Value;

use std::env::args;
use std::fs::File;
use std::collections::HashMap;

fn get_character_id(xivdb: &XivDb, name: &str, server: &str) -> u64 {
  let mut params = HashMap::new();
  params.insert(String::from("one"), String::from("characters"));
  params.insert(String::from("server|et"), server.to_string());
  params.insert(String::from("strict"), String::from("on"));
  xivdb.search(name, params).unwrap().characters.unwrap().results[0]["id"].as_u64().unwrap()
}

fn main() {
  let path = args().nth(1).unwrap();
  let file = File::open(path).unwrap();
  let mut db: Value = serde_json::from_reader(file).unwrap();

  let xivdb = XivDb::default();

  for user in db["autotags"]["users"].as_array_mut().unwrap() {
    let c = get_character_id(&xivdb, user["character"].as_str().unwrap(), user["server"].as_str().unwrap());
    user["character_id"] = c.into();
  }

  println!("{}", serde_json::to_string(&db).unwrap());
}
