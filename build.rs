extern crate git2;

use git2::{Repository, RepositoryState};
use std::fs::File;
use std::path::Path;
use std::io::Write;

fn main() {
  let git = git_commit().map(|x| format!("-g{}", x)).unwrap_or_default();
  let clean = match git_clean() {
    Some(true) => "-dirty",
    _ => ""
  };
  let version = std::env::var("CARGO_PKG_VERSION").unwrap();
  let version_string = format!("{}{}{}", version, git, clean);
  let out_dir = std::env::var("OUT_DIR").unwrap();
  let p = Path::new(&out_dir);
  let mut f = File::create(p.join("version")).unwrap();
  f.write_all(version_string.as_bytes()).unwrap();
}

fn git_commit() -> Option<String> {
  let repo = Repository::open(".").ok()?;
  let revparse = repo.revparse_single("HEAD").ok()?;
  revparse.short_id().ok()?.as_str().map(ToOwned::to_owned)
}

fn git_clean() -> Option<bool> {
  let repo = Repository::open(".").ok()?;
  Some(repo.state() == RepositoryState::Clean)
}
