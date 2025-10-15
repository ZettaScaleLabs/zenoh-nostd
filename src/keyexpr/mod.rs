pub(crate) mod borrowed;
pub(crate) mod canon;
pub(crate) mod include;
pub(crate) mod intersect;

pub(crate) mod utils;

pub(crate) const DELIMITER: u8 = b'/';
pub(crate) const SINGLE_WILD: u8 = b'*';
pub(crate) const DOUBLE_WILD: &[u8] = b"**";
pub(crate) const STAR_DSL: &[u8] = b"$*";

pub(crate) struct ZKeyLength<const LEN: usize>;

crate::__internal_zerr! {
    /// Errors related to key expressions.
    #[err = "keyexpr error"]
    enum ZKeyError {
        KeyExprNotMatch,
        Overflow,
    }
}
