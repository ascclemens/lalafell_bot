use database::schema::*;

use serenity::model::gateway::GameType;

use std::mem;

insertable! {
  #[derive(Debug, Queryable, Identifiable)]
  pub struct Presence,
  #[derive(Debug, Insertable)]
  #[table_name = "presences"]
  pub struct NewPresence {
    pub kind: i16,
    pub content: String
  }
}

impl NewPresence {
  pub fn new(kind: i16, content: &str) -> Self {
    NewPresence {
      kind,
      content: content.to_owned()
    }
  }
}

#[derive(Debug)]
#[repr(i16)]
pub enum PresenceKind {
  Playing,
  Listening
}

impl PresenceKind {
  pub fn as_discord(&self) -> GameType {
    match *self {
      PresenceKind::Playing => GameType::Playing,
      PresenceKind::Listening => GameType::Listening
    }
  }

  pub fn from_i16(i: i16) -> Option<PresenceKind> {
    if i >= PresenceKind::Playing as i16 && i <= PresenceKind::Listening as i16 {
      Some(unsafe { mem::transmute(i) })
    } else {
      None
    }
  }
}

impl ToString for PresenceKind {
  fn to_string(&self) -> String {
    match *self {
      PresenceKind::Playing => "Playing",
      PresenceKind::Listening => "Listening to"
    }.to_string()
  }
}
