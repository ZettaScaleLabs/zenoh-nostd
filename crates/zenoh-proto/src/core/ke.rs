mod borrowed;
pub use borrowed::*;

mod intersect;

const DELIMITER: u8 = b'/';
const SINGLE_WILD: u8 = b'*';
