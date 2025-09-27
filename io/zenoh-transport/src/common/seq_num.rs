use zenoh_protocol::{core::Bits, transport::TransportSn};

const RES_U8: TransportSn = (u8::MAX >> 1) as TransportSn; // 1 byte max when encoded
const RES_U16: TransportSn = (u16::MAX >> 2) as TransportSn; // 2 bytes max when encoded
const RES_U32: TransportSn = (u32::MAX >> 4) as TransportSn; // 4 bytes max when encoded
const RES_U64: TransportSn = (u64::MAX >> 1) as TransportSn; // 9 bytes max when encoded

pub(crate) fn get_mask(resolution: Bits) -> TransportSn {
    match resolution {
        Bits::U8 => RES_U8,
        Bits::U16 => RES_U16,
        Bits::U32 => RES_U32,
        Bits::U64 => RES_U64,
    }
}
