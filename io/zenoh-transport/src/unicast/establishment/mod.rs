pub mod ext;
pub mod open;

use async_trait::async_trait;
use sha3::{
    digest::{ExtendableOutput, Update, XofReader},
    Shake128,
};
use std::boxed::Box;
use zenoh_protocol::{
    core::{Field, Resolution, ZenohIdProto},
    transport::TransportSn,
};

/*************************************/
/*             TRAITS                */
/*************************************/
#[async_trait]
pub trait OpenFsm {
    type Error;

    type SendInitSynIn;
    type SendInitSynOut;
    async fn send_init_syn(
        self,
        input: Self::SendInitSynIn,
    ) -> Result<Self::SendInitSynOut, Self::Error>;

    type RecvInitAckIn;
    type RecvInitAckOut;
    async fn recv_init_ack(
        self,
        input: Self::RecvInitAckIn,
    ) -> Result<Self::RecvInitAckOut, Self::Error>;

    type SendOpenSynIn;
    type SendOpenSynOut;
    async fn send_open_syn(
        self,
        input: Self::SendOpenSynIn,
    ) -> Result<Self::SendOpenSynOut, Self::Error>;

    type RecvOpenAckIn;
    type RecvOpenAckOut;
    async fn recv_open_ack(
        self,
        input: Self::RecvOpenAckIn,
    ) -> Result<Self::RecvOpenAckOut, Self::Error>;
}

#[async_trait]
pub trait AcceptFsm {
    type Error;

    type RecvInitSynIn;
    type RecvInitSynOut;
    async fn recv_init_syn(
        self,
        input: Self::RecvInitSynIn,
    ) -> Result<Self::RecvInitSynOut, Self::Error>;

    type SendInitAckIn;
    type SendInitAckOut;
    async fn send_init_ack(
        self,
        input: Self::SendInitAckIn,
    ) -> Result<Self::SendInitAckOut, Self::Error>;

    type RecvOpenSynIn;
    type RecvOpenSynOut;
    async fn recv_open_syn(
        self,
        input: Self::RecvOpenSynIn,
    ) -> Result<Self::RecvOpenSynOut, Self::Error>;

    type SendOpenAckIn;
    type SendOpenAckOut;
    async fn send_open_ack(
        self,
        input: Self::SendOpenAckIn,
    ) -> Result<Self::SendOpenAckOut, Self::Error>;
}

/*************************************/
/*           FUNCTIONS               */
/*************************************/
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
