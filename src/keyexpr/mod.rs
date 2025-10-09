pub mod borrowed;
pub mod canon;
pub mod include;
pub mod intersect;

pub mod utils;

pub(crate) const DELIMITER: u8 = b'/';
pub(crate) const SINGLE_WILD: u8 = b'*';
pub(crate) const DOUBLE_WILD: &[u8] = b"**";
pub(crate) const STAR_DSL: &[u8] = b"$*";

pub struct ZKeyLength<const LEN: usize>;

crate::__internal_zerr! {
    /// Errors related to key expressions.
    #[err = "keyexpr error"]
    enum ZKeyError {
        KeyExprNotMatch,
        Overflow,
    }
}
