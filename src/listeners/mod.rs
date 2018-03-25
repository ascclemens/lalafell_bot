macro_rules! result_wrap {
  ($(#[$($meta:meta),+])* fn $name:ident($($($kw:ident)+: $ty:ty),*) -> $res:ty $block:block $err:expr) => {
    $(#[$($meta),+])*
    fn $name($($($kw)+: $ty),*) {
      #[allow(unused_mut)]
      let mut inner = || -> $res { $block };
      if let Err(e) = inner() {
        $err(e);
      }
    }
  };
  ($(#[$($meta:meta),+])* fn $name:ident(&$self_:ident, $($($kw:ident)+: $ty:ty),*) -> $res:ty $block:block $err:expr) => {
    $(#[$($meta),+])*
    fn $name(&$self_, $($($kw)+: $ty),*) {
      #[allow(unused_mut)]
      let mut inner = || -> $res { $block };
      if let Err(e) = inner() {
        $err(e);
      }
    }
  };
}

pub mod auto_reply;
pub mod guilds_ext;
pub mod log;
pub mod music;
pub mod party_finder;
pub mod poll_tagger;
pub mod random_presence;
pub mod reaction_authorize;
pub mod temporary_roles;
pub mod timeouts;

pub use self::auto_reply::AutoReplyListener;
pub use self::guilds_ext::GuildsExt;
pub use self::log::Log;
pub use self::music::Music;
pub use self::party_finder::PartyFinder;
pub use self::poll_tagger::PollTagger;
pub use self::random_presence::RandomPresenceListener;
pub use self::reaction_authorize::ReactionAuthorize;
pub use self::temporary_roles::TemporaryRolesListener;
pub use self::timeouts::Timeouts;
