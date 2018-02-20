use serenity::model::channel::ReactionType;
use serenity::model::misc::EmojiIdentifier;

use std::str::FromStr;
use std::num::ParseIntError;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::ops::Deref;

#[derive(Debug, Default)]
pub struct ParsedDuration(pub u64);

impl Deref for ParsedDuration {
  type Target = u64;

  fn deref(&self) -> &u64 {
    &self.0
  }
}

#[derive(Debug)]
pub enum ParsedDurationError {
  Int(ParseIntError),
  Format(String)
}

impl Display for ParsedDurationError {
  fn fmt(&self, f: &mut Formatter) -> FmtResult {
    match *self {
      ParsedDurationError::Int(ref e) => write!(f, "invalid number: {}", e),
      ParsedDurationError::Format(ref s) => write!(f, "invalid duration format: {}", s)
    }
  }
}

impl FromStr for ParsedDuration {
  type Err = ParsedDurationError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    parse_duration_secs(s).map(ParsedDuration)
  }
}

pub fn parse_duration_secs<S: AsRef<str>>(duration: S) -> Result<u64, ParsedDurationError> {
  let duration = duration.as_ref();
  let mut str_length = 0;
  let mut total_time = 0;
  while str_length < duration.len() {
    let numbers: String = duration.chars()
      .skip(str_length)
      .take_while(|c| c.is_numeric())
      .collect();
    str_length += numbers.len();
    let length: u64 = numbers.parse().map_err(ParsedDurationError::Int)?;
    let units: String = duration.chars()
      .skip(str_length)
      .take_while(|c| c.is_alphabetic() || c.is_whitespace())
      .collect();
    str_length += units.len();
    let multiplier = match units.trim().to_lowercase().as_ref() {
      "" if total_time == 0 => 1,
      "s" | "sec" | "secs" | "second" | "seconds" => 1,
      "m" | "min" | "mins" | "minute" | "minutes" => 60,
      "h" | "hr" | "hrs" | "hour" | "hours" => 3600,
      "d" | "ds" | "day" | "days" => 86400,
      _ => return Err(ParsedDurationError::Format("invalid unit".into()))
    };
    total_time += length * multiplier;
  }
  Ok(total_time)
}

#[derive(Debug)]
pub struct ParsedEmoji(pub ReactionType);

impl<'a> From<&'a str> for ParsedEmoji {
  fn from(s: &'a str) -> Self {
    ParsedEmoji(parse_emoji(s))
  }
}

impl Deref for ParsedEmoji {
  type Target = ReactionType;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

pub fn parse_emoji(emoji: &str) -> ReactionType {
  EmojiIdentifier::from_str(emoji)
    .map(ReactionType::from)
    .unwrap_or_else(|_| emoji.into())
}
