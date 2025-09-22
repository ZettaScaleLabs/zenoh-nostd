//
// Copyright (c) 2022 ZettaScale Technology
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ZettaScale Zenoh Team, <zenoh@zettascale.tech>
//
use core::time::Duration;
use std::boxed::Box;

use async_trait::async_trait;
use zenoh_buffers::ZSlice;
use zenoh_link::unicast::LinkUnicast;
use zenoh_protocol::{
    core::{EndPoint, Field, Resolution, WhatAmI, ZenohIdProto},
    transport::{
        batch_size, close, BatchSize, Close, InitSyn, OpenSyn, TransportBody, TransportMessage,
        TransportSn,
    },
    VERSION,
};
use zenoh_result::{zerror, ZResult};

use crate::{
    unicast::{
        establishment::{compute_sn, ext, OpenFsm},
        link::{TransportLinkUnicast, TransportLinkUnicastConfig, TransportLinkUnicastDirection},
    },
    TransportManager,
};

type OpenError = (zenoh_result::Error, Option<u8>);

pub struct StateTransport {
    pub batch_size: BatchSize,
    pub resolution: Resolution,
    pub ext_qos: ext::qos::StateOpen,
    pub ext_lowlatency: ext::lowlatency::StateOpen,
    pub ext_patch: ext::patch::StateOpen,
}

pub struct State {
    pub transport: StateTransport,
}

// InitSyn
pub struct SendInitSynIn {
    pub mine_version: u8,
    pub mine_zid: ZenohIdProto,
    pub mine_whatami: WhatAmI,
}

// InitAck
pub struct RecvInitAckOut {
    pub other_zid: ZenohIdProto,
    pub other_whatami: WhatAmI,
    pub other_cookie: ZSlice,
}

// OpenSyn
pub struct SendOpenSynIn {
    pub mine_zid: ZenohIdProto,
    pub mine_lease: Duration,
    pub other_zid: ZenohIdProto,
    pub other_cookie: ZSlice,
}

pub struct SendOpenSynOut {
    pub mine_initial_sn: TransportSn,
}

// OpenAck
pub struct RecvOpenAckOut {
    pub other_lease: Duration,
    pub other_initial_sn: TransportSn,
}

// FSM
pub struct OpenLink<'a> {
    pub ext_qos: ext::qos::QoSFsm<'a>,
    pub ext_lowlatency: ext::lowlatency::LowLatencyFsm<'a>,
    pub ext_patch: ext::patch::PatchFsm<'a>,
}

#[async_trait]
impl<'a, 'b: 'a> OpenFsm for &'a mut OpenLink<'b> {
    type Error = OpenError;

    type SendInitSynIn = (&'a mut TransportLinkUnicast, &'a mut State, SendInitSynIn);
    type SendInitSynOut = ();
    async fn send_init_syn(
        self,
        input: Self::SendInitSynIn,
    ) -> Result<Self::SendInitSynOut, Self::Error> {
        let (link, state, input) = input;

        // Extension QoS
        let (ext_qos, ext_qos_link) = self
            .ext_qos
            .send_init_syn(&state.transport.ext_qos)
            .await
            .map_err(|e| (e, Some(close::reason::GENERIC)))?;

        // Extension LowLatency
        let ext_lowlatency = self
            .ext_lowlatency
            .send_init_syn(&state.transport.ext_lowlatency)
            .await
            .map_err(|e| (e, Some(close::reason::GENERIC)))?;

        // Extension Patch
        let ext_patch = self
            .ext_patch
            .send_init_syn(&state.transport.ext_patch)
            .await
            .map_err(|e| (e, Some(close::reason::GENERIC)))?;

        let msg: TransportMessage = InitSyn {
            version: input.mine_version,
            whatami: input.mine_whatami,
            zid: input.mine_zid,
            batch_size: state.transport.batch_size,
            resolution: state.transport.resolution,
            ext_qos: ext_qos,
            ext_qos_link: ext_qos_link,
            ext_auth: None,
            ext_mlink: None,
            ext_lowlatency: ext_lowlatency,
            ext_compression: None,
            ext_patch: ext_patch,
        }
        .into();

        let _ = link
            .send(&msg)
            .await
            .map_err(|e| (e, Some(close::reason::GENERIC)))?;

        Ok(())
    }

    type RecvInitAckIn = (&'a mut TransportLinkUnicast, &'a mut State);
    type RecvInitAckOut = RecvInitAckOut;
    async fn recv_init_ack(
        self,
        input: Self::RecvInitAckIn,
    ) -> Result<Self::RecvInitAckOut, Self::Error> {
        let (link, state) = input;

        let msg = link
            .recv()
            .await
            .map_err(|e| (e, Some(close::reason::INVALID)))?;

        let init_ack = match msg.body {
            TransportBody::InitAck(init_ack) => init_ack,
            TransportBody::Close(Close { reason, .. }) => {
                let e = zerror!(
                    "Received a close message (reason {}) in response to an InitSyn",
                    close::reason_to_str(reason),
                );

                return Err((e.into(), None));
            }
            _ => {
                let e = zerror!(
                    "Received an invalid message in response to an InitSyn: {:?}",
                    msg.body
                );

                return Err((e.into(), Some(close::reason::INVALID)));
            }
        };

        // Compute the minimum SN resolution
        state.transport.resolution = {
            let mut res = Resolution::default();

            // Frame SN
            let i_fsn_res = init_ack.resolution.get(Field::FrameSN);
            let m_fsn_res = state.transport.resolution.get(Field::FrameSN);

            if i_fsn_res > m_fsn_res {
                let e = zerror!(
                    "Invalid FrameSN resolution: {:?} > {:?}",
                    i_fsn_res,
                    m_fsn_res
                );

                return Err((e.into(), Some(close::reason::INVALID)));
            }
            res.set(Field::FrameSN, i_fsn_res);

            // Request ID
            let i_rid_res = init_ack.resolution.get(Field::RequestID);
            let m_rid_res = state.transport.resolution.get(Field::RequestID);

            if i_rid_res > m_rid_res {
                let e = zerror!(
                    "Invalid RequestID resolution: {:?} > {:?}",
                    i_rid_res,
                    m_rid_res
                );

                return Err((e.into(), Some(close::reason::INVALID)));
            }
            res.set(Field::RequestID, i_rid_res);

            res
        };

        // Compute the minimum batch size
        state.transport.batch_size = state.transport.batch_size.min(init_ack.batch_size);

        // Extension QoS
        self.ext_qos
            .recv_init_ack((
                &mut state.transport.ext_qos,
                (init_ack.ext_qos, init_ack.ext_qos_link),
            ))
            .await
            .map_err(|e| (e, Some(close::reason::GENERIC)))?;

        // Extension LowLatency
        self.ext_lowlatency
            .recv_init_ack((&mut state.transport.ext_lowlatency, init_ack.ext_lowlatency))
            .await
            .map_err(|e| (e, Some(close::reason::GENERIC)))?;

        // Extension Patch
        self.ext_patch
            .recv_init_ack((&mut state.transport.ext_patch, init_ack.ext_patch))
            .await
            .map_err(|e| (e, Some(close::reason::GENERIC)))?;

        let output = RecvInitAckOut {
            other_zid: init_ack.zid,
            other_whatami: init_ack.whatami,
            other_cookie: init_ack.cookie,
        };
        Ok(output)
    }

    type SendOpenSynIn = (&'a mut TransportLinkUnicast, &'a mut State, SendOpenSynIn);
    type SendOpenSynOut = SendOpenSynOut;
    async fn send_open_syn(
        self,
        input: Self::SendOpenSynIn,
    ) -> Result<Self::SendOpenSynOut, Self::Error> {
        let (link, state, input) = input;

        // Extension QoS
        let ext_qos = self
            .ext_qos
            .send_open_syn(&state.transport.ext_qos)
            .await
            .map_err(|e| (e, Some(close::reason::GENERIC)))?;

        // Extension LowLatency
        let ext_lowlatency = self
            .ext_lowlatency
            .send_open_syn(&state.transport.ext_lowlatency)
            .await
            .map_err(|e| (e, Some(close::reason::GENERIC)))?;

        // Build and send an OpenSyn message
        let mine_initial_sn =
            compute_sn(input.mine_zid, input.other_zid, state.transport.resolution);

        let msg: TransportMessage = OpenSyn {
            lease: input.mine_lease,
            initial_sn: mine_initial_sn,
            cookie: input.other_cookie,
            ext_qos,
            ext_auth: None,
            ext_mlink: None,
            ext_lowlatency,
            ext_compression: None,
        }
        .into();

        let _ = link
            .send(&msg)
            .await
            .map_err(|e| (e, Some(close::reason::GENERIC)))?;

        let output = SendOpenSynOut { mine_initial_sn };
        Ok(output)
    }

    type RecvOpenAckIn = (&'a mut TransportLinkUnicast, &'a mut State);
    type RecvOpenAckOut = RecvOpenAckOut;
    async fn recv_open_ack(
        self,
        input: Self::RecvOpenAckIn,
    ) -> Result<Self::RecvOpenAckOut, Self::Error> {
        let (link, state) = input;

        let msg = link
            .recv()
            .await
            .map_err(|e| (e, Some(close::reason::INVALID)))?;

        let open_ack = match msg.body {
            TransportBody::OpenAck(open_ack) => open_ack,
            TransportBody::Close(Close { reason, .. }) => {
                let e = zerror!(
                    "Received a close message (reason {}) in response to an OpenSyn",
                    close::reason_to_str(reason),
                );

                return Err((e.into(), None));
            }
            _ => {
                let e = zerror!(
                    "Received an invalid message in response to an OpenSyn: {:?}",
                    msg.body
                );

                return Err((e.into(), Some(close::reason::INVALID)));
            }
        };

        // Extension QoS
        self.ext_qos
            .recv_open_ack((&mut state.transport.ext_qos, open_ack.ext_qos))
            .await
            .map_err(|e| (e, Some(close::reason::GENERIC)))?;

        // Extension LowLatency
        self.ext_lowlatency
            .recv_open_ack((&mut state.transport.ext_lowlatency, open_ack.ext_lowlatency))
            .await
            .map_err(|e| (e, Some(close::reason::GENERIC)))?;

        let output = RecvOpenAckOut {
            other_initial_sn: open_ack.initial_sn,
            other_lease: open_ack.lease,
        };
        Ok(output)
    }
}

pub async fn open_link(
    endpoint: &EndPoint,
    link: LinkUnicast,
    tm: &TransportManager,
) -> ZResult<(TransportLinkUnicast, SendOpenSynOut, RecvOpenAckOut)> {
    let direction = TransportLinkUnicastDirection::Outbound;
    let is_streamed = link.is_streamed();

    let config = TransportLinkUnicastConfig {
        direction,
        mtu: link.get_mtu(),
        is_streamed,
    };

    let mut link = TransportLinkUnicast::new(link, config);

    let mut fsm = OpenLink {
        ext_qos: ext::qos::QoSFsm::new(),
        ext_lowlatency: ext::lowlatency::LowLatencyFsm::new(),
        ext_patch: ext::patch::PatchFsm::new(),
    };

    // Clippy raises a warning because `batch_size::UNICAST` is currently equal to `BatchSize::MAX`.
    // However, the current code catches the cases where `batch_size::UNICAST` is different from `BatchSize::MAX`.
    #[allow(clippy::unnecessary_min_or_max)]
    let batch_size = tm.batch_size.min(link.config.mtu).min(batch_size::UNICAST);

    let mut state = {
        State {
            transport: StateTransport {
                batch_size: batch_size,
                resolution: tm.resolution,
                ext_qos: ext::qos::StateOpen::new(tm.unicast.is_qos, &endpoint)?, // TODO shouldn't be false by default
                ext_lowlatency: ext::lowlatency::StateOpen::new(tm.unicast.is_lowlatency), // TODO shouldn't be false by default
                ext_patch: ext::patch::StateOpen::new(),
            },
        }
    };

    // Init handshake
    macro_rules! step {
        ($s: expr) => {
            match $s {
                Ok(output) => output,
                Err((e, _)) => {
                    return Err(e);
                }
            }
        };
    }

    // TODO should be defined elsewhere in the session !!
    let mine_zid = tm.zid;
    let mine_whatami = tm.whatami;
    let mine_lease = tm.unicast.lease;

    let isyn_in = SendInitSynIn {
        mine_version: VERSION,
        mine_zid,
        mine_whatami,
    };

    step!(fsm.send_init_syn((&mut link, &mut state, isyn_in)).await);

    let iack_out = step!(fsm.recv_init_ack((&mut link, &mut state)).await);

    // Open handshake
    let osyn_in = SendOpenSynIn {
        mine_zid,
        other_zid: iack_out.other_zid,
        mine_lease,
        other_cookie: iack_out.other_cookie.clone(),
    };
    let osyn_out = step!(fsm.send_open_syn((&mut link, &mut state, osyn_in)).await);

    let oack_out = step!(fsm.recv_open_ack((&mut link, &mut state)).await);

    let o_config = TransportLinkUnicastConfig {
        direction,
        mtu: state.transport.batch_size,
        is_streamed,
    };

    let o_link = link.reconfigure(o_config);

    Ok((o_link, osyn_out, oack_out))
}
