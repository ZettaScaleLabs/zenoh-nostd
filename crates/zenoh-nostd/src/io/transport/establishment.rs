use sha3::{
    Shake128,
    digest::{ExtendableOutput, Update, XofReader},
};

use zenoh_proto::fields::*;

pub(crate) mod open;

const RES_U8: u32 = (u8::MAX >> 1) as u32; // 1 byte max when encoded
const RES_U16: u32 = (u16::MAX >> 2) as u32; // 2 bytes max when encoded
const RES_U32: u32 = u32::MAX >> 4; // 4 bytes max when encoded
const RES_U64: u32 = (u64::MAX >> 1) as u32; // 9 bytes max when encoded

pub(super) fn compute_sn(zid1: &ZenohIdProto, zid2: &ZenohIdProto, resolution: Resolution) -> u32 {
    let mut hasher = Shake128::default();
    hasher.update(&zid1.as_le_bytes()[..zid1.size()]);
    hasher.update(&zid2.as_le_bytes()[..zid2.size()]);
    let mut array = 0_u32.to_le_bytes();
    hasher.finalize_xof().read(&mut array);
    u32::from_le_bytes(array) & get_mask(resolution.get(Field::FrameSN))
}

fn get_mask(resolution: Bits) -> u32 {
    match resolution {
        Bits::U8 => RES_U8,
        Bits::U16 => RES_U16,
        Bits::U32 => RES_U32,
        Bits::U64 => RES_U64,
    }
}
