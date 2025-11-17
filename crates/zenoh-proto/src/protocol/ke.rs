mod borrowed;
pub use borrowed::*;

mod intersect;

const DELIMITER: u8 = b'/';
const SINGLE_WILD: u8 = b'*';

crate::make_zerr! {
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
