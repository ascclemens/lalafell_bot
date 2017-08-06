#![allow(unused_doc_comment)]

use xivdb;
use lalafell;

error_chain! {
  links {
    XivDb(xivdb::error::Error, xivdb::error::ErrorKind);
    Lalafell(lalafell::error::Error, lalafell::error::ErrorKind);
  }
}
