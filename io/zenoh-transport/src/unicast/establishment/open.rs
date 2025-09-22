use core::time::Duration;

use zenoh_buffers::ZSlice;
use zenoh_link::unicast::LinkUnicast;
use zenoh_protocol::{
    core::{Field, Resolution, WhatAmI, ZenohIdProto},
    transport::{
        batch_size, close, ext::PatchType, BatchSize, Close, InitSyn, OpenSyn, TransportBody,
        TransportMessage, TransportSn,
    },
    VERSION,
};

use zenoh_result::{zerror, ZResult};

use crate::{
    unicast::{
        establishment::compute_sn,
        link::{TransportLinkUnicast, TransportLinkUnicastConfig},
    },
    TransportManager,
};

pub struct StateTransport {
    pub batch_size: BatchSize,
    pub resolution: Resolution,
}

// InitSyn
pub struct SendInitSynIn {
    pub mine_version: u8,
    pub mine_zid: ZenohIdProto,
    pub mine_whatami: WhatAmI,
}

impl SendInitSynIn {
    pub async fn send(
        &self,
        link: &mut TransportLinkUnicast,
        state: &StateTransport,
    ) -> ZResult<()> {
        let msg: TransportMessage = InitSyn {
            version: self.mine_version,
            whatami: self.mine_whatami,
            zid: self.mine_zid,
            batch_size: state.batch_size,
            resolution: state.resolution,
            ext_qos: None,
            ext_qos_link: None,
            ext_auth: None,
            ext_mlink: None,
            ext_lowlatency: None,
            ext_compression: None,
            ext_patch: PatchType::CURRENT,
        }
        .into();

        let _ = link.send(&msg).await?;

        Ok(())
    }
}

// InitAck
pub struct RecvInitAckOut {
    pub other_zid: ZenohIdProto,
    pub other_whatami: WhatAmI,
    pub other_cookie: ZSlice,
}

impl RecvInitAckOut {
    pub async fn recv(
        link: &mut TransportLinkUnicast,
        state: &mut StateTransport,
    ) -> ZResult<Self> {
        let msg = link.recv().await?;

        let init_ack = match msg.body {
            TransportBody::InitAck(init_ack) => init_ack,
            TransportBody::Close(Close { reason, .. }) => {
                let e = zerror!(
                    "Received a close message (reason {}) in response to an InitSyn",
                    close::reason_to_str(reason),
                );

                return Err(e.into());
            }
            _ => {
                let e = zerror!(
                    "Received an invalid message in response to an InitSyn: {:?}",
                    msg.body
                );

                return Err(e.into());
            }
        };

        state.resolution = {
            let mut res = Resolution::default();

            let i_fsn_res = init_ack.resolution.get(Field::FrameSN);
            let m_fsn_res = state.resolution.get(Field::FrameSN);

            if i_fsn_res > m_fsn_res {
                let e = zerror!(
                    "Invalid FrameSN resolution: {:?} > {:?}",
                    i_fsn_res,
                    m_fsn_res
                );

                return Err(e.into());
            }
            res.set(Field::FrameSN, i_fsn_res);

            let i_rid_res = init_ack.resolution.get(Field::RequestID);
            let m_rid_res = state.resolution.get(Field::RequestID);

            if i_rid_res > m_rid_res {
                let e = zerror!(
                    "Invalid RequestID resolution: {:?} > {:?}",
                    i_rid_res,
                    m_rid_res
                );

                return Err(e.into());
            }
            res.set(Field::RequestID, i_rid_res);

            res
        };

        state.batch_size = state.batch_size.min(init_ack.batch_size);

        let output = RecvInitAckOut {
            other_zid: init_ack.zid,
            other_whatami: init_ack.whatami,
            other_cookie: init_ack.cookie,
        };

        Ok(output)
    }
}

// OpenSyn
pub struct SendOpenSynIn {
    pub mine_zid: ZenohIdProto,
    pub mine_lease: Duration,
    pub other_zid: ZenohIdProto,
    pub other_cookie: ZSlice,
}

impl SendOpenSynIn {
    pub async fn send(
        &self,
        link: &mut TransportLinkUnicast,
        state: &StateTransport,
    ) -> ZResult<SendOpenSynOut> {
        let mine_initial_sn = compute_sn(self.mine_zid, self.other_zid, state.resolution);

        let msg: TransportMessage = OpenSyn {
            lease: self.mine_lease,
            initial_sn: mine_initial_sn,
            cookie: self.other_cookie.clone(),
            ext_qos: None,
            ext_auth: None,
            ext_mlink: None,
            ext_lowlatency: None,
            ext_compression: None,
        }
        .into();

        let _ = link.send(&msg).await?;

        let output = SendOpenSynOut { mine_initial_sn };

        Ok(output)
    }
}

pub struct SendOpenSynOut {
    pub mine_initial_sn: TransportSn,
}

// OpenAck
pub struct RecvOpenAckOut {
    pub other_lease: Duration,
    pub other_initial_sn: TransportSn,
}

impl RecvOpenAckOut {
    pub async fn recv(link: &mut TransportLinkUnicast) -> ZResult<Self> {
        let msg = link.recv().await?;

        let open_ack = match msg.body {
            TransportBody::OpenAck(open_ack) => open_ack,
            TransportBody::Close(Close { reason, .. }) => {
                let e = zerror!(
                    "Received a close message (reason {}) in response to an OpenSyn",
                    close::reason_to_str(reason),
                );

                return Err(e.into());
            }
            _ => {
                let e = zerror!(
                    "Received an invalid message in response to an OpenSyn: {:?}",
                    msg.body
                );

                return Err(e.into());
            }
        };

        let output = RecvOpenAckOut {
            other_initial_sn: open_ack.initial_sn,
            other_lease: open_ack.lease,
        };

        Ok(output)
    }
}

pub async fn open_link(
    link: LinkUnicast,
    tm: &TransportManager,
) -> ZResult<(TransportLinkUnicast, SendOpenSynOut, RecvOpenAckOut)> {
    let is_streamed = link.is_streamed();

    let config = TransportLinkUnicastConfig {
        mtu: link.get_mtu(),
        is_streamed,
    };

    let mut link = TransportLinkUnicast::new(link, config);

    #[allow(clippy::unnecessary_min_or_max)]
    let batch_size = tm.batch_size.min(link.config.mtu).min(batch_size::UNICAST);

    let mut state = StateTransport {
        batch_size,
        resolution: tm.resolution,
    };

    let isyn_in = SendInitSynIn {
        mine_version: VERSION,
        mine_zid: tm.zid,
        mine_whatami: tm.whatami,
    };

    isyn_in.send(&mut link, &state).await?;
    let iack_out = RecvInitAckOut::recv(&mut link, &mut state).await?;

    // Open handshake
    let osyn_in = SendOpenSynIn {
        mine_zid: tm.zid,
        other_zid: iack_out.other_zid,
        mine_lease: tm.unicast.lease,
        other_cookie: iack_out.other_cookie.clone(),
    };
    let osyn_out = osyn_in.send(&mut link, &state).await?;
    let oack_out = RecvOpenAckOut::recv(&mut link).await?;

    let o_config = TransportLinkUnicastConfig {
        mtu: state.batch_size,
        is_streamed,
    };

    let o_link = link.reconfigure(o_config);

    Ok((o_link, osyn_out, oack_out))
}
