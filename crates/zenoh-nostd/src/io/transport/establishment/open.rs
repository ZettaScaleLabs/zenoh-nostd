use ::core::time::Duration;

use crate::io::{
    link::ZLink,
    transport::{
        Transport, TransportConfig, TransportMineConfig, TransportNegociatedConfig,
        TransportOtherConfig, ZTransportRecv, ZTransportSend, establishment::compute_sn,
    },
};
use zenoh_proto::{exts::Patch, fields::*, msgs::*, zbail, *};

pub(crate) struct StateTransport {
    pub(crate) batch_size: u16,
    pub(crate) resolution: Resolution,
}

pub(crate) struct SendInitSynIn {
    pub(crate) mine_version: u8,
    pub(crate) mine_zid: ZenohIdProto,
    pub(crate) mine_whatami: WhatAmI,
}

impl SendInitSynIn {
    pub(crate) async fn send(
        &self,
        tx: &mut impl AsMut<[u8]>,
        transport: &mut impl ZTransportSend,
        state: &StateTransport,
    ) -> ZResult<(), crate::ZTransportError> {
        let msg = InitSyn {
            version: self.mine_version,
            identifier: InitIdentifier {
                whatami: self.mine_whatami,
                zid: self.mine_zid.clone(),
            },
            resolution: InitResolution {
                resolution: state.resolution,
                batch_size: BatchSize(state.batch_size),
            },
            patch: Patch::current(),
            ..Default::default()
        };

        transport
            .send(tx.as_mut(), &mut 0, |batch| batch.unframe(&msg))
            .await
    }
}

pub(crate) struct RecvInitAckOut<'a> {
    pub(crate) other_zid: ZenohIdProto,
    pub(crate) other_whatami: WhatAmI,
    pub(crate) other_cookie: &'a [u8],
}

impl<'a> RecvInitAckOut<'a> {
    pub(crate) async fn recv(
        rx: &'a mut impl AsMut<[u8]>,
        transport: &mut impl ZTransportRecv,
        state: &mut StateTransport,
    ) -> ZResult<Self, crate::ZTransportError> {
        let reader = transport.recv(rx.as_mut()).await?;
        let mut batch = ZBatchReader::new(reader);
        let init_ack = loop {
            match batch.next() {
                Some(ZMessage::InitAck(i)) => break i,
                Some(_) => continue,
                None => zbail!(crate::ZTransportError::InvalidRx),
            }
        };

        state.resolution = {
            let mut res = Resolution::default();

            let i_fsn_res = init_ack.resolution.resolution.get(Field::FrameSN);
            let m_fsn_res = state.resolution.get(Field::FrameSN);

            if i_fsn_res > m_fsn_res {
                zbail!(crate::ZTransportError::InvalidRx);
            }

            res.set(Field::FrameSN, i_fsn_res);

            let i_rid_res = init_ack.resolution.resolution.get(Field::RequestID);
            let m_rid_res = state.resolution.get(Field::RequestID);

            if i_rid_res > m_rid_res {
                zbail!(crate::ZTransportError::InvalidRx);
            }

            res.set(Field::RequestID, i_rid_res);

            res
        };

        state.batch_size = state.batch_size.min(init_ack.resolution.batch_size.0);

        let output = RecvInitAckOut {
            other_zid: init_ack.identifier.zid,
            other_whatami: init_ack.identifier.whatami,
            other_cookie: init_ack.cookie,
        };

        Ok(output)
    }
}

pub(crate) struct SendOpenSynIn<'a> {
    pub(crate) mine_zid: ZenohIdProto,
    pub(crate) mine_lease: Duration,
    pub(crate) other_zid: ZenohIdProto,
    pub(crate) other_cookie: &'a [u8],
}

impl<'a> SendOpenSynIn<'a> {
    pub(crate) async fn send(
        &self,
        tx: &mut impl AsMut<[u8]>,
        transport: &mut impl ZTransportSend,
        state: &StateTransport,
    ) -> ZResult<SendOpenSynOut, crate::ZTransportError> {
        let mine_initial_sn = compute_sn(&self.mine_zid, &self.other_zid, state.resolution);

        let msg = OpenSyn {
            lease: self.mine_lease,
            sn: mine_initial_sn,
            cookie: self.other_cookie,
            ..Default::default()
        };

        transport
            .send(tx.as_mut(), &mut 0, |batch| batch.unframe(&msg))
            .await?;

        let output = SendOpenSynOut {
            mine_sn: mine_initial_sn,
        };

        Ok(output)
    }
}

pub(crate) struct SendOpenSynOut {
    pub(crate) mine_sn: u32,
}

pub(crate) struct RecvOpenAckOut {
    pub(crate) other_lease: Duration,
    #[allow(dead_code)]
    pub(crate) other_sn: u32,
}

impl RecvOpenAckOut {
    pub(crate) async fn recv(
        rx: &mut impl AsMut<[u8]>,
        transport: &mut impl ZTransportRecv,
    ) -> ZResult<Self, crate::ZTransportError> {
        let reader = transport.recv(rx.as_mut()).await?;
        let mut batch = ZBatchReader::new(reader);
        let open_ack = loop {
            match batch.next() {
                Some(ZMessage::OpenAck(i)) => break i,
                Some(_) => continue,
                None => zbail!(crate::ZTransportError::InvalidRx),
            }
        };
        let output = RecvOpenAckOut {
            other_sn: open_ack.sn,
            other_lease: open_ack.lease,
        };

        Ok(output)
    }
}

pub(crate) async fn open_link<T: ZLink>(
    link: T,
    config: TransportMineConfig,
    tx: &mut impl AsMut<[u8]>,
    rx: &mut impl AsMut<[u8]>,
) -> ZResult<(Transport<T>, TransportConfig), crate::ZTransportError> {
    let batch_size = link.mtu().min(rx.as_mut().len() as u16);

    let mut transport = Transport { link };

    let mut state = StateTransport {
        batch_size,
        resolution: Resolution::default(),
    };

    let isyn_in = SendInitSynIn {
        mine_version: zenoh_proto::VERSION,
        mine_zid: config.mine_zid.clone(),
        mine_whatami: WhatAmI::Client,
    };

    isyn_in.send(tx, &mut transport, &state).await?;
    let iack_out = RecvInitAckOut::recv(rx, &mut transport, &mut state).await?;

    let other_zid = iack_out.other_zid.clone();
    let other_whatami = iack_out.other_whatami;

    let osyn_in = SendOpenSynIn {
        mine_zid: config.mine_zid.clone(),
        other_zid: iack_out.other_zid,
        mine_lease: config.mine_lease.try_into().map_err(|e| {
            crate::error!(
                "could not parse mine_lease duration {:?}: {:?}",
                config.mine_lease,
                e
            );
            ZTransportError::CouldNotParseField
        })?,
        other_cookie: iack_out.other_cookie,
    };

    let osyn_out = osyn_in.send(tx, &mut transport, &state).await?;
    let oack_out = RecvOpenAckOut::recv(rx, &mut transport).await?;

    Ok((
        transport,
        TransportConfig {
            mine_config: config,
            other_config: TransportOtherConfig {
                other_zid,
                other_whatami,
                other_sn: osyn_out.mine_sn,
                other_lease: oack_out.other_lease.try_into().map_err(|e| {
                    crate::error!(
                        "could not parse other_lease duration {:?}: {:?}",
                        oack_out.other_lease,
                        e
                    );
                    ZTransportError::CouldNotParseField
                })?,
            },
            negociated_config: TransportNegociatedConfig {
                mine_sn: osyn_out.mine_sn,
                batch_size: state.batch_size,
                resolution: state.resolution,
            },
        },
    ))
}
