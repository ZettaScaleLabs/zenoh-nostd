use sha3::{
    Shake128,
    digest::{ExtendableOutput, Update, XofReader},
};

use crate::protocol::{
    core::{
        ZenohIdProto,
        resolution::{Bits, Field, Resolution},
    },
    transport::TransportSn,
};

pub(crate) mod open;

const RES_U8: TransportSn = (u8::MAX >> 1) as TransportSn; // 1 byte max when encoded
const RES_U16: TransportSn = (u16::MAX >> 2) as TransportSn; // 2 bytes max when encoded
const RES_U32: TransportSn = (u32::MAX >> 4) as TransportSn; // 4 bytes max when encoded
const RES_U64: TransportSn = (u64::MAX >> 1) as TransportSn; // 9 bytes max when encoded

pub(super) fn compute_sn(
    zid1: ZenohIdProto,
    zid2: ZenohIdProto,
    resolution: Resolution,
) -> TransportSn {
    let mut hasher = Shake128::default();
    hasher.update(&zid1.as_le_bytes()[..zid1.size()]);
    hasher.update(&zid2.as_le_bytes()[..zid2.size()]);
    let mut array = (0 as TransportSn).to_le_bytes();
    hasher.finalize_xof().read(&mut array);
    TransportSn::from_le_bytes(array) & get_mask(resolution.get(Field::FrameSN))
}

fn get_mask(resolution: Bits) -> TransportSn {
    match resolution {
        Bits::U8 => RES_U8,
        Bits::U16 => RES_U16,
        Bits::U32 => RES_U32,
        Bits::U64 => RES_U64,
    }
}
