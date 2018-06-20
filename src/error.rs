use xivdb;
use lalafell;
use serenity;

error_chain! {
  links {
    XivDb(xivdb::error::Error, xivdb::error::ErrorKind);
    Lalafell(lalafell::error::Error, lalafell::error::ErrorKind);
  }

  foreign_links {
    Serenity(serenity::Error);
  }
}
