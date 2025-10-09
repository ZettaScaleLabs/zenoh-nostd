use core::time::Duration;

use crate::{
    io::{
        link::Link,
        transport::{
            SingleLinkTransport, SingleLinkTransportConfig, SingleLinkTransportMineConfig,
            SingleLinkTransportNegociatedConfig, SingleLinkTransportOtherConfig,
            establishment::compute_sn,
        },
    },
    platform::{Platform, ZCommunicationError},
    protocol::{
        VERSION,
        core::{
            ZenohIdProto,
            resolution::{Field, Resolution},
            whatami::WhatAmI,
        },
        network::NetworkMessage,
        transport::{
            BatchSize, TransportBody, TransportMessage, TransportSn,
            ext::PatchType,
            frame::FrameHeader,
            init::{InitAck, InitSyn},
            open::{OpenAck, OpenSyn},
        },
    },
    result::ZResult,
    zbail,
    zbuf::{ZBuf, ZBufMut},
};

pub struct StateTransport {
    pub batch_size: BatchSize,
    pub resolution: Resolution,
}

pub struct SendInitSynIn {
    pub mine_version: u8,
    pub mine_zid: ZenohIdProto,
    pub mine_whatami: WhatAmI,
}

impl SendInitSynIn {
    pub async fn send<T: Platform>(
        &self,
        tx_zbuf: ZBufMut<'_>,
        link: &mut SingleLinkTransport<T>,
        state: &StateTransport,
    ) -> ZResult<(), ZCommunicationError> {
        let msg = TransportMessage {
            body: TransportBody::InitSyn(InitSyn {
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
            }),
        };

        link.send(tx_zbuf, &msg).await
    }
}

pub struct RecvInitAckOut<'a> {
    pub other_zid: ZenohIdProto,
    pub other_whatami: WhatAmI,
    pub other_cookie: ZBuf<'a>,
}

impl<'a> RecvInitAckOut<'a> {
    pub async fn recv<T: Platform>(
        rx_zbuf: ZBufMut<'a>,
        link: &mut SingleLinkTransport<T>,
        state: &mut StateTransport,
    ) -> ZResult<Self, ZCommunicationError> {
        let mut reader = link.recv(rx_zbuf).await?;

        let mut init_ack = Option::<InitAck<'a>>::None;
        TransportMessage::decode_batch(
            &mut reader,
            Some(|_: InitSyn| zbail!(ZCommunicationError::Invalid.into())),
            Some(|i: InitAck<'a>| {
                init_ack = Some(i);
                Ok(())
            }),
            Some(|_: OpenSyn| zbail!(ZCommunicationError::Invalid.into())),
            Some(|_: OpenAck| zbail!(ZCommunicationError::Invalid.into())),
            Some(|| zbail!(ZCommunicationError::Invalid.into())),
            Some(|_: &FrameHeader, _: NetworkMessage| zbail!(ZCommunicationError::Invalid.into())),
        )?;

        let Some(init_ack) = init_ack else {
            zbail!(ZCommunicationError::Invalid);
        };

        state.resolution = {
            let mut res = Resolution::default();

            let i_fsn_res = init_ack.resolution.get(Field::FrameSN);
            let m_fsn_res = state.resolution.get(Field::FrameSN);

            if i_fsn_res > m_fsn_res {
                zbail!(ZCommunicationError::Invalid);
            }

            res.set(Field::FrameSN, i_fsn_res);

            let i_rid_res = init_ack.resolution.get(Field::RequestID);
            let m_rid_res = state.resolution.get(Field::RequestID);

            if i_rid_res > m_rid_res {
                zbail!(ZCommunicationError::Invalid);
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

pub struct SendOpenSynIn<'a> {
    pub mine_zid: ZenohIdProto,
    pub mine_lease: Duration,
    pub other_zid: ZenohIdProto,
    pub other_cookie: ZBuf<'a>,
}

impl<'a> SendOpenSynIn<'a> {
    pub async fn send<T: Platform>(
        &self,
        tx_zbuf: ZBufMut<'_>,
        link: &mut SingleLinkTransport<T>,
        state: &StateTransport,
    ) -> ZResult<SendOpenSynOut, ZCommunicationError> {
        let mine_initial_sn = compute_sn(self.mine_zid, self.other_zid, state.resolution);

        let msg = TransportMessage {
            body: TransportBody::OpenSyn(OpenSyn {
                lease: self.mine_lease,
                initial_sn: mine_initial_sn,
                cookie: self.other_cookie,
                ext_qos: None,
                ext_auth: None,
                ext_mlink: None,
                ext_lowlatency: None,
                ext_compression: None,
            }),
        };

        link.send(tx_zbuf, &msg).await?;

        let output = SendOpenSynOut { mine_initial_sn };

        Ok(output)
    }
}

pub struct SendOpenSynOut {
    pub mine_initial_sn: TransportSn,
}

pub struct RecvOpenAckOut {
    pub other_lease: Duration,
    pub other_initial_sn: TransportSn,
}

impl RecvOpenAckOut {
    pub async fn recv<'a, T: Platform>(
        rx_zbuf: ZBufMut<'a>,
        link: &mut SingleLinkTransport<T>,
    ) -> ZResult<Self, ZCommunicationError> {
        let mut reader = link.recv(rx_zbuf).await?;

        let mut msg = Option::<OpenAck<'a>>::None;

        TransportMessage::decode_batch(
            &mut reader,
            Some(|_: InitSyn| zbail!(ZCommunicationError::Invalid.into())),
            Some(|_: InitAck| zbail!(ZCommunicationError::Invalid.into())),
            Some(|_: OpenSyn| zbail!(ZCommunicationError::Invalid.into())),
            Some(|o: OpenAck<'a>| {
                msg = Some(o);
                Ok(())
            }),
            Some(|| zbail!(ZCommunicationError::Invalid.into())),
            Some(|_: &FrameHeader, _: NetworkMessage| zbail!(ZCommunicationError::Invalid.into())),
        )?;

        let Some(open_ack) = msg else {
            zbail!(ZCommunicationError::Invalid);
        };

        let output = RecvOpenAckOut {
            other_initial_sn: open_ack.initial_sn,
            other_lease: open_ack.lease,
        };

        Ok(output)
    }
}

pub async fn open_link<T: Platform, const TX: usize, const RX: usize>(
    link: Link<T>,
    config: SingleLinkTransportMineConfig,
) -> ZResult<(SingleLinkTransport<T>, SingleLinkTransportConfig), ZCommunicationError> {
    let mut tx_zbuf = [0u8; TX];
    let mut rx_zbuf = [0u8; RX];

    let batch_size = link.get_mtu();

    let mut link = SingleLinkTransport::new(link);

    let mut state = StateTransport {
        batch_size,
        resolution: Resolution::default(),
    };

    let isyn_in = SendInitSynIn {
        mine_version: VERSION,
        mine_zid: config.mine_zid,
        mine_whatami: WhatAmI::Client,
    };

    isyn_in.send::<_>(&mut tx_zbuf, &mut link, &state).await?;
    let iack_out = RecvInitAckOut::recv::<_>(&mut rx_zbuf, &mut link, &mut state).await?;

    let other_zid = iack_out.other_zid;
    let other_whatami = iack_out.other_whatami;

    let osyn_in = SendOpenSynIn {
        mine_zid: config.mine_zid,
        other_zid: iack_out.other_zid,
        mine_lease: config.mine_lease,
        other_cookie: iack_out.other_cookie,
    };

    let osyn_out = osyn_in.send::<_>(&mut tx_zbuf, &mut link, &state).await?;
    let oack_out = RecvOpenAckOut::recv::<_>(&mut rx_zbuf, &mut link).await?;

    Ok((
        link,
        SingleLinkTransportConfig {
            mine_config: config,
            other_config: SingleLinkTransportOtherConfig {
                other_zid,
                other_whatami,
                other_sn: osyn_out.mine_initial_sn,
                other_lease: oack_out.other_lease,
            },
            negociated_config: SingleLinkTransportNegociatedConfig {
                mine_sn: osyn_out.mine_initial_sn,
                batch_size: state.batch_size,
                resolution: state.resolution,
            },
        },
    ))
}
