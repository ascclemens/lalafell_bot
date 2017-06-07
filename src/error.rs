use xivdb;

error_chain! {
  links {
    XivDb(xivdb::error::Error, xivdb::error::ErrorKind);
  }
}
