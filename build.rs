extern crate git2;

use git2::{Repository, StatusOptions};
use std::fs::File;
use std::path::Path;
use std::io::Write;

fn main() {
  let repo = Repo(Repository::open(".").ok());
  let git = repo.commit().map(|x| format!("-g{}", x)).unwrap_or_default();
  let branch = repo.branch().map(|x| format!("-{}", x)).unwrap_or_default();
  let clean = match repo.clean() {
    Some(false) => "-dirty",
    _ => ""
  };
  let version = std::env::var("CARGO_PKG_VERSION").unwrap();
  let version_string = format!("{}{}{}{}", version, git, branch, clean);
  let out_dir = std::env::var("OUT_DIR").unwrap();
  let p = Path::new(&out_dir);
  let mut f = File::create(p.join("version")).unwrap();
  f.write_all(version_string.as_bytes()).unwrap();
}

struct Repo(Option<Repository>);

impl Repo {
  fn commit(&self) -> Option<String> {
    let repo = self.0.as_ref()?;
    let revparse = repo.revparse_single("HEAD").ok()?;
    revparse.short_id().ok()?.as_str().map(ToOwned::to_owned)
  }

  fn clean(&self) -> Option<bool> {
    let repo = self.0.as_ref()?;
    let mut options = StatusOptions::new();
    options
      .include_ignored(false)
      .include_untracked(false);
    let statuses = repo.statuses(Some(&mut options)).ok()?;
    Some(statuses.is_empty())
  }

  fn branch(&self) -> Option<String> {
    let repo = self.0.as_ref()?;
    let head = repo.head().ok()?;
    head.name().and_then(|x| {
      if x.starts_with("refs/heads/") {
        Some(x[11..].to_string())
      } else if x.len() >= 7 {
        Some(x[..7].to_string())
      } else {
        None
      }
    })
  }
}
