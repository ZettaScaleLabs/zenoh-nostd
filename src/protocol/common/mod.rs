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

    pub(crate) const fn flags(header: u8) -> u8 {
        header & !HEADER_MASK
    }

    pub(crate) const fn has_flag(byte: u8, flag: u8) -> bool {
        byte & flag != 0
    }

    pub(crate) const fn unset_flag(mut byte: u8, flag: u8) -> u8 {
        byte &= !flag;
        byte
    }

    pub(crate) const fn set_flag(mut byte: u8, flag: u8) -> u8 {
        byte = unset_flag(byte, flag);
        byte |= flag;
        byte
    }

    pub(crate) const fn set_bitfield(mut byte: u8, value: u8, mask: u8) -> u8 {
        byte = unset_flag(byte, mask);
        byte |= value;
        byte
    }

    pub(crate) const fn has_option(options: u64, flag: u64) -> bool {
        options & flag != 0
    }
}
