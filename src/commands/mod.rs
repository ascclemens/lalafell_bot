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
pub mod view_tag;
pub mod verify;
pub mod reference_count;
pub mod timeout;
pub mod archive;
pub mod image_dump;
pub mod random_reaction;
pub mod search;
pub mod presence;
pub mod ping;
pub mod reload_config;
pub mod blob;
pub mod mention;

pub use self::tag::{TagCommand, AutoTagCommand, UpdateTagsCommand, UpdateTagCommand};
pub use self::polling::{PollCommand, PollResultsCommand};
pub use self::config::ConfigureCommand;
pub use self::timeout::{TimeoutCommand, UntimeoutCommand};
pub use self::archive::ArchiveCommand;
pub use self::image_dump::ImageDumpCommand;
pub use self::random_reaction::RandomReactionCommand;
pub use self::search::SearchCommand;
pub use self::presence::PresenceCommand;
pub use self::ping::PingCommand;
pub use self::race::RaceCommand;
pub use self::view_tag::ViewTagCommand;
pub use self::verify::VerifyCommand;
pub use self::reference_count::ReferenceCountCommand;
pub use self::reload_config::ReloadConfigCommand;
pub use self::blob::BlobCommand;
pub use self::mention::MentionCommand;
