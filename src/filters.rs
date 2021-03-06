use lalafell::commands::MentionOrId;

use serenity::model::guild::{Member, Role};

use unicase::UniCase;

use std::borrow::Borrow;
use std::str::FromStr;

pub enum Filter {
  Include(FilterKind),
  Exclude(FilterKind),
}

impl Filter {
  fn find_all_filters(input: &str) -> Option<Vec<String>> {
    let mut roles = Vec::new();
    let mut last_index = 0;
    loop {
      let index = input[last_index..].find("role:").or_else(|| input[last_index..].find("user:"));
      let mut i = some_or!(index, break);
      if i + last_index != 0 && &input[i + last_index - 1..i + last_index] == "!" {
        i -= 1;
      }
      if !input[last_index..last_index + i].trim().is_empty() {
        return None;
      }
      if let Some((bytes, role)) = Filter::lexical_parse(&input[last_index + i..]) {
        last_index += bytes;
        roles.push(role);
      }
      last_index += i;
    }
    if input.len() - last_index != 0 {
      None
    } else {
      Some(roles)
    }
  }

  fn lexical_parse(input: &str) -> Option<(usize, String)> {
    let mut consumed_bytes = 0;
    let mut acc = String::new();
    let mut escaped = false;
    let mut take_whitespace = false;
    for c in input.chars() {
      if c == '`' {
        consumed_bytes += c.len_utf8();
        if take_whitespace {
          break;
        } else {
          take_whitespace = true;
          continue;
        }
      }
      if c == '\\' && !escaped {
        consumed_bytes += c.len_utf8();
        escaped = true;
        continue;
      }
      if escaped {
        acc.push(c);
        consumed_bytes += c.len_utf8();
        escaped = false;
        continue;
      }
      if !take_whitespace && c == ' ' {
        break;
      }
      acc.push(c);
      consumed_bytes += c.len_utf8();
    }
    if acc.is_empty() {
      None
    } else {
      Some((consumed_bytes, acc))
    }
  }

  pub fn all_filters(s: &str) -> Option<Vec<Filter>> {
    Filter::find_all_filters(s)?
      .into_iter()
      .map(|x| Filter::parse(&x))
      .collect()
  }

  pub fn parse(s: &str) -> Option<Filter> {
    if s.starts_with('!') {
      FilterKind::parse(&s[1..]).map(Filter::Exclude)
    } else {
      FilterKind::parse(s).map(Filter::Include)
    }
  }

  pub fn matches<I: Borrow<Role>>(&self, member: &Member, roles: &[I]) -> bool {
    let (include, fk) = match *self {
      Filter::Include(ref fk) => (true, fk),
      Filter::Exclude(ref fk) => (false, fk)
    };
    let roles: Vec<&Role> = roles.iter().map(Borrow::borrow).collect();
    match *fk {
      FilterKind::Role(ref role_name) => {
        let role_name = UniCase::new(role_name);
        let role = match roles.iter().find(|r| UniCase::new(&r.name) == role_name) {
          Some(r) => r,
          None => return !include,
        };
        member.roles.iter().any(|r| *r == role.id) == include
      },
      FilterKind::User(id) => (member.user.read().id.0 == id) == include,
    }
  }
}

impl ToString for Filter {
  fn to_string(&self) -> String {
    match *self {
      Filter::Include(ref fk) => fk.to_string(),
      Filter::Exclude(ref fk) => format!("!{}", fk.to_string()),
    }
  }
}

pub enum FilterKind {
  Role(String),
  User(u64),
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
        let id = MentionOrId::from_str(value).ok()?;
        Some(FilterKind::User(id.0))
      },
      _ => None,
    }
  }
}

impl ToString for FilterKind {
  fn to_string(&self) -> String {
    match *self {
      FilterKind::Role(ref role) if role.contains(' ') => format!("role:`{}`", role),
      FilterKind::Role(ref role) => format!("role:{}", role),
      FilterKind::User(id) => format!("user:{}", id),
    }
  }
}
