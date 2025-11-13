pub(crate) mod borrowed;
pub use borrowed::*;

pub(crate) mod intersect;

pub(crate) const DELIMITER: u8 = b'/';
pub(crate) const SINGLE_WILD: u8 = b'*';

crate::__internal_zerr! {
    /// Errors related to key expressions.
    #[err = "keyexpr error"]
    enum ZKeyExprError {
        LoneDollarStar,
        SingleStarAfterDoubleStar,
        DoubleStarAfterDoubleStar,
        EmptyChunk,
        StarInChunk,
        DollarAfterDollar,
        SharpOrQMark,
        UnboundDollar,
        WildChunk,
    }
}
