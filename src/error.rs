use lalafell;
use serenity;

error_chain! {
  links {
    Lalafell(lalafell::error::Error, lalafell::error::ErrorKind);
  }

  foreign_links {
    Serenity(serenity::Error);
  }
}
