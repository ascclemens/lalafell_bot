pub use lalafell::commands::{MentionOrId, ChannelOrId};

pub mod archive;
pub mod blob;
pub mod bot;
pub mod config;
pub mod ephemeral_message;
pub mod fflogs;
pub mod image_dump;
pub mod mention;
pub mod ping;
pub mod polling;
pub mod race;
pub mod random_reaction;
pub mod reference_count;
pub mod reload_config;
pub mod report;
pub mod search;
pub mod tag;
pub mod temporary_role;
pub mod timeout;
pub mod verify;
pub mod version;
pub mod view_tag;

pub use self::archive::ArchiveCommand;
pub use self::blob::BlobCommand;
pub use self::bot::BotCommand as ActualBotCommand;
pub use self::config::ConfigureCommand;
pub use self::ephemeral_message::EphemeralMessageCommand;
pub use self::fflogs::FfLogsCommand;
pub use self::image_dump::ImageDumpCommand;
pub use self::mention::MentionCommand;
pub use self::ping::PingCommand;
pub use self::polling::{PollCommand, PollResultsCommand};
pub use self::race::RaceCommand;
pub use self::random_reaction::RandomReactionCommand;
pub use self::reference_count::ReferenceCountCommand;
pub use self::reload_config::ReloadConfigCommand;
pub use self::report::ReportCommand;
pub use self::search::SearchCommand;
pub use self::tag::{TagCommand, AutoTagCommand, QueueTagCommand, UpdateTagsCommand, UpdateTagCommand};
pub use self::temporary_role::TemporaryRoleCommand;
pub use self::timeout::{TimeoutCommand, UntimeoutCommand};
pub use self::verify::VerifyCommand;
pub use self::version::VersionCommand;
pub use self::view_tag::ViewTagCommand;

use crate::bot::BotEnv;

use std::sync::Arc;

pub trait BotCommand {
  fn new(_: Arc<BotEnv>) -> Self;
}
