use sha3::{
    digest::{ExtendableOutput, Update, XofReader},
    Shake128,
};

use zenoh_protocol::{
    core::{Field, Resolution, ZenohIdProto},
    transport::TransportSn,
};

pub mod open;

pub(super) fn compute_sn(
    zid1: ZenohIdProto,
    zid2: ZenohIdProto,
    resolution: Resolution,
) -> TransportSn {
    // Create a random yet deterministic initial_sn.
    // In case of multilink it's important that the same initial_sn is used for every connection attempt.
    // Instead of storing the state everywhere, we make sure that the we always compute the same initial_sn.
    let mut hasher = Shake128::default();
    hasher.update(&zid1.to_le_bytes()[..zid1.size()]);
    hasher.update(&zid2.to_le_bytes()[..zid2.size()]);
    let mut array = (0 as TransportSn).to_le_bytes();
    hasher.finalize_xof().read(&mut array);
    TransportSn::from_le_bytes(array)
        & crate::common::seq_num::get_mask(resolution.get(Field::FrameSN))
}
