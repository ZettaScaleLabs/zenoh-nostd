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
use alloc::boxed::Box;
use core::time::Duration;
use zenoh_link_commons::LinkUnicast;

use async_trait::async_trait;
use zenoh_buffers::ZSlice;
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
    common::BatchConfig,
    unicast::{
        establishment::{compute_sn, ext, OpenFsm},
        link::{TransportLinkUnicast, TransportLinkUnicastConfig, TransportLinkUnicastDirection},
    },
};

type OpenError = (zenoh_result::Error, Option<u8>);

struct StateTransport {
    batch_size: BatchSize,
    resolution: Resolution,
    ext_qos: ext::qos::StateOpen,
    ext_lowlatency: ext::lowlatency::StateOpen,
    ext_patch: ext::patch::StateOpen,
}

struct State {
    transport: StateTransport,
}

// InitSyn
struct SendInitSynIn {
    mine_version: u8,
    mine_zid: ZenohIdProto,
    mine_whatami: WhatAmI,
}

// InitAck
struct RecvInitAckOut {
    other_zid: ZenohIdProto,
    other_whatami: WhatAmI,
    other_cookie: ZSlice,
}

// OpenSyn
struct SendOpenSynIn {
    mine_zid: ZenohIdProto,
    mine_lease: Duration,
    other_zid: ZenohIdProto,
    other_cookie: ZSlice,
}

struct SendOpenSynOut {
    mine_initial_sn: TransportSn,
}

// OpenAck
struct RecvOpenAckOut {
    other_lease: Duration,
    other_initial_sn: TransportSn,
}

// FSM
struct OpenLink<'a> {
    ext_qos: ext::qos::QoSFsm<'a>,
    ext_lowlatency: ext::lowlatency::LowLatencyFsm<'a>,
    ext_patch: ext::patch::PatchFsm<'a>,
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
                    "Received a close message (reason {}) in response to an InitSyn on: {}",
                    close::reason_to_str(reason),
                    link,
                );

                return Err((e.into(), None));
            }
            _ => {
                let e = zerror!(
                    "Received an invalid message in response to an InitSyn on {}: {:?}",
                    link,
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
                    "Invalid FrameSN resolution on {}: {:?} > {:?}",
                    link,
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
                    "Invalid RequestID resolution on {}: {:?} > {:?}",
                    link,
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
                    "Received a close message (reason {}) in response to an OpenSyn on: {:?}",
                    close::reason_to_str(reason),
                    link,
                );

                return Err((e.into(), None));
            }
            _ => {
                let e = zerror!(
                    "Received an invalid message in response to an OpenSyn on {}: {:?}",
                    link,
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

pub async fn open_link(endpoint: EndPoint, link: LinkUnicast) -> ZResult<TransportLinkUnicast> {
    let direction = TransportLinkUnicastDirection::Outbound;
    let is_streamed = link.is_streamed();
    let config = TransportLinkUnicastConfig {
        direction,
        batch: BatchConfig {
            mtu: link.get_mtu(),
            is_streamed,
        },
        priorities: None,
        reliability: None,
        sn_resolution: None,
        tx_initial_sn: None,
        zid: None,
        whatami: None,
        mine_lease: None,
        other_lease: None,
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
    let batch_size = batch_size::UNICAST.min(link.config.batch.mtu);

    let mut state = {
        State {
            transport: StateTransport {
                batch_size,
                resolution: Resolution::default(),
                ext_qos: ext::qos::StateOpen::new(false, &endpoint)?, // TODO shouldn't be false by default
                ext_lowlatency: ext::lowlatency::StateOpen::new(false), // TODO shouldn't be false by default
                ext_patch: ext::patch::StateOpen::new(),
            },
        }
    };

    // Init handshake
    macro_rules! step {
        ($s: expr) => {
            match $s {
                Ok(output) => output,
                Err((e, reason)) => {
                    let _ = link.close(reason).await;
                    return Err(e);
                }
            }
        };
    }

    // TODO should be defined elsewhere in the session !!
    let mine_zid = ZenohIdProto::rand();
    let mine_whatami = WhatAmI::Client;
    let mine_lease = Duration::from_secs(10);

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
        other_cookie: iack_out.other_cookie,
    };
    let osyn_out = step!(fsm.send_open_syn((&mut link, &mut state, osyn_in)).await);

    let oack_out = step!(fsm.recv_open_ack((&mut link, &mut state)).await);

    let o_config = TransportLinkUnicastConfig {
        direction,
        batch: BatchConfig {
            mtu: state.transport.batch_size,
            is_streamed,
        },
        priorities: state.transport.ext_qos.priorities(),
        reliability: state.transport.ext_qos.reliability(),
        zid: Some(mine_zid),
        whatami: Some(mine_whatami),
        sn_resolution: Some(state.transport.resolution.get(Field::FrameSN)),
        tx_initial_sn: Some(osyn_out.mine_initial_sn),
        mine_lease: Some(mine_lease),
        other_lease: Some(oack_out.other_lease),
    };

    let o_link = link.reconfigure(o_config);

    Ok(o_link)
}
