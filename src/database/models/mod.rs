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

// NOTE: No need for this module yet, but if it ever arises, this is useful to have.
// pub mod administrators;
pub mod auto_replies;
pub mod config;
pub mod ephemeral_messages;
pub mod delete_all_messages;
pub mod log_channels;
pub mod presences;
pub mod role_check_times;
pub mod roles;
pub mod tags;
pub mod temporary_roles;
pub mod tag_queue;
pub mod timeouts;
pub mod verifications;

// pub use self::administrators::{Administrator, NewAdministrator};
pub use self::auto_replies::{AutoReply, NewAutoReply};
pub use self::config::{ServerConfig, NewServerConfig, ChannelConfig, NewChannelConfig, Reaction, NewReaction, PartyFinderConfig, NewPartyFinderConfig};
pub use self::ephemeral_messages::{EphemeralMessage, NewEphemeralMessage};
pub use self::delete_all_messages::{DeleteAllMessages, NewDeleteAllMessages};
pub use self::log_channels::{LogChannel, NewLogChannel};
pub use self::presences::{Presence, NewPresence, PresenceKind};
pub use self::role_check_times::{RoleCheckTime, NewRoleCheckTime};
pub use self::roles::{Role, NewRole};
pub use self::tags::{Tag, NewTag};
pub use self::temporary_roles::{TemporaryRole, NewTemporaryRole};
pub use self::tag_queue::{TagQueue, NewTagQueue};
pub use self::timeouts::{Timeout, NewTimeout};
pub use self::verifications::{Verification, NewVerification};

use serenity::model::id::{UserId, GuildId, ChannelId, MessageId, RoleId, EmojiId};

use std::ops::Deref;
use std::error::Error;
use std::fmt::{Display, Formatter, Error as FmtError};

use byteorder::ReadBytesExt;

use diesel::pg::Pg;
use diesel::row::Row;
use diesel::backend::Backend;
use diesel::query_source::Queryable;
use diesel::expression::AsExpression;
use diesel::expression::helper_types::AsExprOf;
use diesel::types::{FromSql, FromSqlRow, HasSqlType};
use diesel::sql_types::BigInt;

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

#[derive(Debug, Clone, Copy)]
pub struct U64(u64);

impl<DB> Queryable<BigInt, DB> for U64
  where DB: Backend + HasSqlType<BigInt>,
        U64: FromSql<BigInt, DB>
{
  type Row = Self;

  fn build(row: Self::Row) -> Self {
    row
  }
}

impl FromSql<BigInt, Pg> for U64 {
  fn from_sql(bytes: Option<&<Pg as Backend>::RawValue>) -> Result<Self, Box<Error + Send + Sync>> {
    let mut bytes = match bytes {
      Some(b) => b,
      None => return Err(box SqlError::new("unexpected null"))
    };
    bytes
      .read_u64::<<Pg as Backend>::ByteOrder>()
      .map(U64)
      .map_err(|e| Box::new(e) as Box<Error + Send + Sync>)
  }
}

impl<DB> FromSqlRow<BigInt, DB> for U64
  where DB: Backend + HasSqlType<BigInt>,
        U64: FromSql<BigInt, DB>
{
  fn build_from_row<T: Row<DB>>(row: &mut T) -> Result<Self, Box<Error + Send + Sync>> {
    FromSql::from_sql(row.take())
  }
}

impl<'a> AsExpression<BigInt> for &'a U64 {
  type Expression = AsExprOf<i64, BigInt>;

  fn as_expression(self) -> Self::Expression {
    AsExpression::<BigInt>::as_expression(self.0 as i64)
  }
}

impl AsExpression<BigInt> for U64 {
  type Expression = AsExprOf<i64, BigInt>;

  fn as_expression(self) -> Self::Expression {
    AsExpression::<BigInt>::as_expression(self.0 as i64)
  }
}

impl Deref for U64 {
  type Target = u64;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl From<u64> for U64 {
  fn from(u: u64) -> Self {
    U64(u)
  }
}

impl From<U64> for u64 {
  fn from(u: U64) -> Self {
    *u
  }
}

pub trait ToU64 {
  fn to_u64(self) -> U64;
}

macro_rules! from {
  ($($name: ident;)*) => {
    $(
      impl From<$name> for U64 {
        fn from(u: $name) -> U64 {
          U64(u.0)
        }
      }

      impl<'a> ToU64 for &'a $name {
        fn to_u64(self) -> U64 {
          U64(self.0)
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
