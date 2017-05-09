use term::{self, Terminal, TerminfoTerminal};
use term::terminfo::TermInfo;
use fern;
use log::{LogLevel, LogLevelFilter};
use std::io::{self, Write};
use chrono;

use xivdb::error::*;

lazy_static! {
  static ref TERM_INFO: Option<TermInfo> = TermInfo::from_env().ok();
}

fn colored_level(level: LogLevel) -> String {
  let mut t = match *TERM_INFO {
    Some(ref t) => TerminfoTerminal::new_with_terminfo(vec![0u8, 0], t.clone()),
    None => return format!("{}", level)
  };
  let color = match level {
    LogLevel::Trace => term::color::BRIGHT_BLACK,
    LogLevel::Info => term::color::BLUE,
    LogLevel::Warn => term::color::YELLOW,
    LogLevel::Error => term::color::RED,
    _ => return format!("{}", level)
  };
  t.fg(color).unwrap();
  write!(t, "{}", level).unwrap();
  t.reset().unwrap();
  String::from_utf8_lossy(&t.into_inner()).to_string()
}

fn colored_target(target: &str) -> String {
  let parts: Vec<&str> = target.split("::").collect();
  if parts.len() == 1 {
    return target.to_string();
  }
  let base = &parts[..parts.len() - 1];
  let target = &parts[parts.len() - 1];
  let mut t = match *TERM_INFO {
    Some(ref t) => TerminfoTerminal::new_with_terminfo(vec![0u8, 0], t.clone()),
    None => return target.to_string()
  };
  for part in base {
    write!(t, "{}", part).unwrap();
    t.fg(term::color::BRIGHT_BLACK).unwrap();
    write!(t, "::").unwrap();
    t.reset().unwrap();
  }
  write!(t, "{}", target).unwrap();
  String::from_utf8_lossy(&t.into_inner()).to_string()
}

pub fn init_logger() -> Result<()> {
  fern::Dispatch::new()
    .format(|out, message, record| {
      out.finish(format_args!("[{}] [{}] {} – {}",
        chrono::Local::now().format("%H:%M:%S"),
        colored_level(record.level()),
        colored_target(record.target()),
        message))
    })
    .level(LogLevelFilter::Info)
    .chain(io::stdout())
    .apply()
    .chain_err(|| "could not set up logger")
}
