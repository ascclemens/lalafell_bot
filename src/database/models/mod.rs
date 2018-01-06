macro_rules! insertable {
  ($(#[$($meta:meta),+])* pub struct $name:ident, $(#[$($new_meta:meta),+])* pub struct $new_name:ident {
    $(pub $field_name:ident: $kind:ty),+
  }) => {
    $(#[$($meta),+])*
    pub struct $name {
      pub id: i32,
      $(pub $field_name: $kind),+
    }

    $(#[$($new_meta),+])*
    pub struct $new_name {
      $(pub $field_name: $kind),+
    }
  }
}

pub mod tags;
pub mod verifications;
pub mod timeouts;
pub mod messages;
pub mod edits;
pub mod config;
pub mod auto_replies;
pub mod delete_all_messages;

pub use self::tags::{Tag, NewTag};
pub use self::verifications::{Verification, NewVerification};
pub use self::timeouts::{Timeout, NewTimeout};
pub use self::messages::{Message, NewMessage};
pub use self::edits::{Edit, NewEdit};
pub use self::config::{ServerConfig, NewServerConfig, ChannelConfig, NewChannelConfig, Reaction, NewReaction};
pub use self::auto_replies::{AutoReply, NewAutoReply};
pub use self::delete_all_messages::{DeleteAllMessages, NewDeleteAllMessages};

use serenity::model::id::{UserId, GuildId, ChannelId, MessageId, RoleId, EmojiId};

use std::error::Error;
use std::ops::Deref;
use std::fmt::{Display, Formatter, Error as FmtError};

use diesel::types::{FromSql, FromSqlRow, HasSqlType, Text};
use diesel::query_source::Queryable;
use diesel::expression::AsExpression;
use diesel::expression::helper_types::AsExprOf;
use diesel::backend::Backend;
use diesel::row::Row;
use diesel::pg::Pg;

#[derive(Debug)]
struct SqlError(String);

impl SqlError {
  fn new<S: AsRef<str>>(s: S) -> Self {
    SqlError(s.as_ref().to_string())
  }
}

impl Display for SqlError {
  fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
    write!(f, "{}", self.0)
  }
}

impl Error for SqlError {
  fn description(&self) -> &str {
    "there was an sql error"
  }

  fn cause(&self) -> Option<&Error> {
    None
  }
}

#[derive(Debug)]
pub struct U64(u64);

impl<DB> Queryable<Text, DB> for U64
  where DB: Backend + HasSqlType<Text>,
        U64: FromSql<Text, DB>
{
  type Row = Self;

  fn build(row: Self::Row) -> Self {
    row
  }
}

impl FromSql<Text, Pg> for U64 {
  fn from_sql(bytes: Option<&<Pg as Backend>::RawValue>) -> Result<Self, Box<Error + Send + Sync>> {
    let bytes = match bytes {
      Some(b) => b,
      None => return Err(box SqlError::new("unexpected null"))
    };
    let string = String::from_utf8(bytes.to_vec()).map_err(Box::new)?;
    let u = string.parse::<u64>().map_err(Box::new)?;
    Ok(U64(u))
  }
}

impl<DB> FromSqlRow<Text, DB> for U64
  where DB: Backend + HasSqlType<Text>,
        U64: FromSql<Text, DB>
{
  fn build_from_row<T: Row<DB>>(row: &mut T) -> Result<Self, Box<Error + Send + Sync>> {
    FromSql::from_sql(row.take())
  }
}

impl<'a> AsExpression<Text> for &'a U64 {
  type Expression = AsExprOf<String, Text>;

  fn as_expression(self) -> Self::Expression {
    AsExpression::<Text>::as_expression(self.0.to_string())
  }
}

impl Deref for U64 {
  type Target = u64;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl From<u64> for U64 {
  fn from(u: u64) -> U64 {
    U64(u)
  }
}

macro_rules! from {
  ($($name: ident;)*) => {
    $(
      impl From<$name> for U64 {
        fn from(u: $name) -> U64 {
          U64(u.0)
        }
      }
    )*
  }
}

from! {
	UserId;
	GuildId;
	ChannelId;
	MessageId;
	RoleId;
	EmojiId;
}
