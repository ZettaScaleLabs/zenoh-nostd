pub(crate) const DELIMITER: u8 = b'/';
pub(crate) const SINGLE_WILD: u8 = b'*';
pub(crate) const DOUBLE_WILD: &[u8] = b"**";
pub(crate) const STAR_DSL: &[u8] = b"$*";

pub(crate) mod borrowed;
pub mod canon;
pub mod include;
pub mod intersect;
pub(crate) mod owned;
pub(crate) mod utils;

pub use borrowed::*;
pub use owned::{OwnedKeyExpr, OwnedNonWildKeyExpr};

pub struct ZKeyLength<const LEN: usize>;
