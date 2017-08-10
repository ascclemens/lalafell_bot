use lalafell::commands::MentionOrId;

use discord::model::{Member, Role};

pub enum Filter {
  Include(FilterKind),
  Exclude(FilterKind)
}

impl Filter {
  pub fn parse(s: &str) -> Option<Filter> {
    if s.starts_with('!') {
      let fk = match FilterKind::parse(&s[1..]) {
        Some(f) => f,
        None => return None
      };
      Some(Filter::Exclude(fk))
    } else {
      let fk = match FilterKind::parse(s) {
        Some(f) => f,
        None => return None
      };
      Some(Filter::Include(fk))
    }
  }

  pub fn matches(&self, member: &Member, roles: &[Role]) -> bool {
    let (include, fk) = match *self {
      Filter::Include(ref fk) => (true, fk),
      Filter::Exclude(ref fk) => (false, fk)
    };
    match *fk {
      FilterKind::Role(ref role_name) => {
        let role_name = role_name.to_lowercase();
        let role = match roles.iter().find(|r| r.name.to_lowercase() == role_name) {
          Some(r) => r,
          None => return !include
        };
        member.roles.iter().any(|r| *r == role.id) == include
      }
      FilterKind::User(id) => (member.user.id.0 == id) == include
    }
  }
}

impl ToString for Filter {
  fn to_string(&self) -> String {
    let (start, fk) = match *self {
      Filter::Include(ref fk) => ("", fk),
      Filter::Exclude(ref fk) => ("!", fk)
    };
    format!("{}{}", start, fk.to_string())
  }
}

pub enum FilterKind {
  Role(String),
  User(u64)
}

impl FilterKind {
  pub fn parse(s: &str) -> Option<FilterKind> {
    let parts: Vec<&str> = s.splitn(2, ':').collect();
    if parts.len() != 2 {
      return None;
    }
    let (kind, value) = (&parts[0], &parts[1]);
    match kind.to_lowercase().as_str() {
      "role" => Some(FilterKind::Role(value.to_string())),
      "user" | "member" => {
        let id = match MentionOrId::parse(value) {
          Ok(i) => i,
          Err(_) => return None
        };
        Some(FilterKind::User(id.0))
      }
      _ => None
    }
  }
}

impl ToString for FilterKind {
  fn to_string(&self) -> String {
    match *self {
      FilterKind::Role(ref role) => format!("role:{}", role),
      FilterKind::User(id) => format!("user:{}", id)
    }
  }
}
