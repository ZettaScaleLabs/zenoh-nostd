pub(crate) const DELIMITER: u8 = b'/';
pub(crate) const SINGLE_WILD: u8 = b'*';
pub(crate) const DOUBLE_WILD: &[u8] = b"**";
pub(crate) const STAR_DSL: &[u8] = b"$*";

pub(crate) mod owned;
pub use owned::{OwnedKeyExpr, OwnedNonWildKeyExpr};

pub(crate) mod borrowed;
pub use borrowed::*;

/// Used to implement and expose the tools to implement canonization of Key Expressions for string-like types.
/// The average user doesn't need to bother with it.
pub mod canon;
/// Used to implement and expose the tools to implement algorithms to detect Key Expression inclusivity.
/// The average user doesn't need to bother with it.
pub mod include;
/// Used to implement and expose the tools to implement algorithms to detect Key Expression intersection.
/// The average user doesn't need to bother with it.
pub mod intersect;
pub(crate) mod utils;

// pub mod format;

// #[cfg(test)]
// mod tests;
