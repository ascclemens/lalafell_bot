pub mod tags;
pub mod verifications;
pub mod timeouts;

pub use self::tags::{Tag, NewTag};
pub use self::verifications::{Verification, NewVerification};
pub use self::timeouts::{Timeout, NewTimeout};

use std::error::Error;
use std::ops::Deref;
use std::fmt::{Display, Formatter, Error as FmtError};

use diesel::types::{FromSql, FromSqlRow, HasSqlType, Text};
use diesel::backend::Backend;
use diesel::row::Row;

#[derive(Debug)]
pub struct U64(u64);

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

use diesel::sqlite::Sqlite;

impl FromSql<Text, Sqlite> for U64 {
  fn from_sql(bytes: Option<&<Sqlite as Backend>::RawValue>) -> Result<Self, Box<Error + Send + Sync>> {
    let bytes = match bytes {
      Some(b) => b,
      None => return Err(box SqlError::new("unexpected null"))
    };
    let u = bytes.read_text().parse::<u64>().map_err(Box::new)?;
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

impl Deref for U64 {
  type Target = u64;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}
