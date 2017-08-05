pub use lalafell::commands::{MentionOrId, ChannelOrId};

macro_rules! into {
  ($t:ty, $e:expr) => {{
    let x: $t = $e.into();
    x
  }}
}

pub mod tag;
pub mod polling;
pub mod config;

pub mod race;
pub mod viewtag;
pub mod verify;
pub mod reference_count;
pub mod timeout;
pub mod archive;
pub mod view_edits;
pub mod image_dump;
pub mod random_reaction;

pub use self::tag::{TagCommand, AutoTagCommand, UpdateTagsCommand, UpdateTagCommand};
pub use self::polling::{PollCommand, PollResultsCommand};
pub use self::config::ConfigureCommand;

pub use self::timeout::{TimeoutCommand, UntimeoutCommand};
pub use self::archive::ArchiveCommand;
pub use self::view_edits::ViewEditsCommand;
pub use self::image_dump::ImageDumpCommand;
pub use self::random_reaction::RandomReactionCommand;

pub use self::race::RaceCommand;
pub use self::viewtag::ViewTagCommand;
pub use self::verify::VerifyCommand;
pub use self::reference_count::ReferenceCountCommand;
