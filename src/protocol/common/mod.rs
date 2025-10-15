pub(crate) mod extension;

/*************************************/
/*               IDS                 */
/*************************************/

pub(crate) mod imsg {

    pub(crate) const HEADER_BITS: u8 = 5;
    pub(crate) const HEADER_MASK: u8 = !(0xff << HEADER_BITS);

    pub(crate) const fn mid(header: u8) -> u8 {
        header & HEADER_MASK
    }

    pub(crate) const fn has_flag(byte: u8, flag: u8) -> bool {
        byte & flag != 0
    }
}
