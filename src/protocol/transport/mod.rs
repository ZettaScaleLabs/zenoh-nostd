pub(crate) mod frame;
pub(crate) mod init;
pub(crate) mod keepalive;
pub(crate) mod open;

use core::fmt;

use crate::{
    protocol::{
        ZCodecError,
        common::imsg,
        network::NetworkMessage,
        transport::{
            self,
            frame::{Frame, FrameHeader},
            init::{InitAck, InitSyn},
            keepalive::KeepAlive,
            open::{OpenAck, OpenSyn},
        },
    },
    result::ZResult,
    zbuf::{BufReaderExt, ZBufReader, ZBufWriter},
};

pub(crate) type BatchSize = u16;

pub(crate) mod batch_size {
    use super::BatchSize;

    pub(crate) const UNICAST: BatchSize = BatchSize::MAX;
}

pub(crate) mod id {
    pub(crate) const INIT: u8 = 0x01;
    pub(crate) const OPEN: u8 = 0x02;
    pub(crate) const KEEP_ALIVE: u8 = 0x04;
    pub(crate) const FRAME: u8 = 0x05;
}

pub(crate) type TransportSn = u32;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum TransportBody<'a, 'b> {
    InitSyn(InitSyn<'a>),
    #[allow(dead_code)]
    InitAck(InitAck<'a>),
    OpenSyn(OpenSyn<'a>),
    #[allow(dead_code)]
    OpenAck(OpenAck<'a>),
    KeepAlive(KeepAlive),
    Frame(Frame<'a, 'b>),
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct TransportMessage<'a, 'b> {
    pub(crate) body: TransportBody<'a, 'b>,
}

impl<'a, 'b> TransportMessage<'a, 'b> {
    pub(crate) fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
        match &self.body {
            TransportBody::InitSyn(m) => m.encode(writer),
            TransportBody::InitAck(m) => m.encode(writer),
            TransportBody::OpenSyn(m) => m.encode(writer),
            TransportBody::OpenAck(m) => m.encode(writer),
            TransportBody::KeepAlive(m) => m.encode(writer),
            TransportBody::Frame(m) => m.encode(writer),
        }
    }

    fn handle_frame_callback(
        header: u8,
        reader: &mut ZBufReader<'a>,
        on_network_msg: &mut Option<impl FnMut(&FrameHeader, NetworkMessage<'a>) -> ZResult<()>>,
    ) -> ZResult<(), ZCodecError> {
        let header: FrameHeader = FrameHeader::decode(header, reader)?;

        while reader.can_read() {
            let mark = reader.mark();
            let msg = NetworkMessage::decode(header.reliability, reader);

            match msg {
                Ok(msg) => {
                    if let Some(on_network_msg) = on_network_msg
                        && let Err(e) = on_network_msg(&header, msg)
                    {
                        crate::error!("Error when handling network_msg callback: {}", e);
                    }
                }
                Err(_) => {
                    reader.rewind(mark);
                    break;
                }
            }
        }

        Ok(())
    }

    pub(crate) fn decode_single(
        reader: &mut ZBufReader<'a>,
        mut on_init_syn: Option<impl FnMut(InitSyn<'a>) -> ZResult<()>>,
        mut on_init_ack: Option<impl FnMut(InitAck<'a>) -> ZResult<()>>,
        mut on_open_syn: Option<impl FnMut(OpenSyn<'a>) -> ZResult<()>>,
        mut on_open_ack: Option<impl FnMut(OpenAck<'a>) -> ZResult<()>>,
        mut on_keepalive: Option<impl FnMut() -> ZResult<()>>,
        mut on_network_msg: Option<impl FnMut(&FrameHeader, NetworkMessage<'a>) -> ZResult<()>>,
    ) -> ZResult<bool, ZCodecError> {
        let Ok(header): ZResult<u8, ZCodecError> = crate::protocol::zcodec::decode_u8(reader)
        else {
            return Ok(false);
        };

        match imsg::mid(header) {
            id::FRAME => {
                Self::handle_frame_callback(header, reader, &mut on_network_msg)?;
            }
            id::KEEP_ALIVE => {
                let _ = KeepAlive::decode(header, reader)?;
                if let Some(on_keepalive) = &mut on_keepalive
                    && let Err(e) = on_keepalive()
                {
                    crate::error!("Error when handling keepalive callback: {}", e);
                }
            }
            id::INIT => {
                if !imsg::has_flag(header, transport::init::flag::A) {
                    let init_syn = InitSyn::decode(header, reader)?;
                    if let Some(on_init_syn) = &mut on_init_syn
                        && let Err(e) = on_init_syn(init_syn)
                    {
                        crate::error!("Error when handling init_syn callback: {}", e);
                    }
                } else {
                    let init_ack = InitAck::decode(header, reader)?;
                    if let Some(on_init_ack) = &mut on_init_ack
                        && let Err(e) = on_init_ack(init_ack)
                    {
                        crate::error!("Error when handling init_ack callback: {}", e);
                    }
                }
            }
            id::OPEN => {
                if !imsg::has_flag(header, transport::open::flag::A) {
                    let open_syn = OpenSyn::decode(header, reader)?;
                    if let Some(on_open_syn) = &mut on_open_syn
                        && let Err(e) = on_open_syn(open_syn)
                    {
                        crate::error!("Error when handling open_syn callback: {}", e);
                    }
                } else {
                    let open_ack = OpenAck::decode(header, reader)?;
                    if let Some(on_open_ack) = &mut on_open_ack
                        && let Err(e) = on_open_ack(open_ack)
                    {
                        crate::error!("Error when handling open_ack callback: {}", e);
                    }
                }
            }
            _ => {
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub(crate) fn decode_batch(
        reader: &mut ZBufReader<'a>,
        mut on_init_syn: Option<impl FnMut(InitSyn<'a>) -> ZResult<()>>,
        mut on_init_ack: Option<impl FnMut(InitAck<'a>) -> ZResult<()>>,
        mut on_open_syn: Option<impl FnMut(OpenSyn<'a>) -> ZResult<()>>,
        mut on_open_ack: Option<impl FnMut(OpenAck<'a>) -> ZResult<()>>,
        mut on_keepalive: Option<impl FnMut() -> ZResult<()>>,
        mut on_network_msg: Option<impl FnMut(&FrameHeader, NetworkMessage<'a>) -> ZResult<()>>,
    ) -> ZResult<(), ZCodecError> {
        while reader.can_read() {
            let cont = Self::decode_single(
                reader,
                on_init_syn.as_mut(),
                on_init_ack.as_mut(),
                on_open_syn.as_mut(),
                on_open_ack.as_mut(),
                on_keepalive.as_mut(),
                on_network_msg.as_mut(),
            )?;

            if !cont {
                break;
            }
        }

        Ok(())
    }

    async fn handle_frame_callback_async(
        header: u8,
        reader: &mut ZBufReader<'a>,
        on_network_msg: &mut Option<
            impl AsyncFnMut(&FrameHeader, NetworkMessage<'a>) -> ZResult<()>,
        >,
    ) -> ZResult<(), ZCodecError> {
        let header: FrameHeader = FrameHeader::decode(header, reader)?;

        while reader.can_read() {
            let mark = reader.mark();
            let msg = NetworkMessage::decode(header.reliability, reader);

            match msg {
                Ok(msg) => {
                    if let Some(on_network_msg) = on_network_msg
                        && let Err(e) = on_network_msg(&header, msg).await
                    {
                        crate::error!("Error when handling network_msg callback: {}", e);
                    }
                }
                Err(_) => {
                    reader.rewind(mark);
                    break;
                }
            }
        }

        Ok(())
    }

    pub(crate) async fn decode_single_async(
        reader: &mut ZBufReader<'a>,
        mut on_init_syn: Option<impl FnMut(InitSyn<'a>) -> ZResult<()>>,
        mut on_init_ack: Option<impl FnMut(InitAck<'a>) -> ZResult<()>>,
        mut on_open_syn: Option<impl FnMut(OpenSyn<'a>) -> ZResult<()>>,
        mut on_open_ack: Option<impl FnMut(OpenAck<'a>) -> ZResult<()>>,
        mut on_keepalive: Option<impl FnMut() -> ZResult<()>>,
        mut on_network_msg: Option<
            impl AsyncFnMut(&FrameHeader, NetworkMessage<'a>) -> ZResult<()>,
        >,
    ) -> ZResult<bool, ZCodecError> {
        let Ok(header): ZResult<u8, ZCodecError> = crate::protocol::zcodec::decode_u8(reader)
        else {
            return Ok(false);
        };

        match imsg::mid(header) {
            id::FRAME => {
                Self::handle_frame_callback_async(header, reader, &mut on_network_msg).await?;
            }
            id::KEEP_ALIVE => {
                let _ = KeepAlive::decode(header, reader)?;
                if let Some(on_keepalive) = &mut on_keepalive
                    && let Err(e) = on_keepalive()
                {
                    crate::error!("Error when handling keepalive callback: {}", e);
                }
            }
            id::INIT => {
                if !imsg::has_flag(header, transport::init::flag::A) {
                    let init_syn = InitSyn::decode(header, reader)?;
                    if let Some(on_init_syn) = &mut on_init_syn
                        && let Err(e) = on_init_syn(init_syn)
                    {
                        crate::error!("Error when handling init_syn callback: {}", e);
                    }
                } else {
                    let init_ack = InitAck::decode(header, reader)?;
                    if let Some(on_init_ack) = &mut on_init_ack
                        && let Err(e) = on_init_ack(init_ack)
                    {
                        crate::error!("Error when handling init_ack callback: {}", e);
                    }
                }
            }
            id::OPEN => {
                if !imsg::has_flag(header, transport::open::flag::A) {
                    let open_syn = OpenSyn::decode(header, reader)?;
                    if let Some(on_open_syn) = &mut on_open_syn
                        && let Err(e) = on_open_syn(open_syn)
                    {
                        crate::error!("Error when handling open_syn callback: {}", e);
                    }
                } else {
                    let open_ack = OpenAck::decode(header, reader)?;
                    if let Some(on_open_ack) = &mut on_open_ack
                        && let Err(e) = on_open_ack(open_ack)
                    {
                        crate::error!("Error when handling open_ack callback: {}", e);
                    }
                }
            }
            _ => {
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub(crate) async fn decode_batch_async(
        reader: &mut ZBufReader<'a>,
        mut on_init_syn: Option<impl FnMut(InitSyn<'a>) -> ZResult<()>>,
        mut on_init_ack: Option<impl FnMut(InitAck<'a>) -> ZResult<()>>,
        mut on_open_syn: Option<impl FnMut(OpenSyn<'a>) -> ZResult<()>>,
        mut on_open_ack: Option<impl FnMut(OpenAck<'a>) -> ZResult<()>>,
        mut on_keepalive: Option<impl FnMut() -> ZResult<()>>,
        mut on_network_msg: Option<
            impl AsyncFnMut(&FrameHeader, NetworkMessage<'a>) -> ZResult<()>,
        >,
    ) -> ZResult<(), ZCodecError> {
        while reader.can_read() {
            let cont = Self::decode_single_async(
                reader,
                on_init_syn.as_mut(),
                on_init_ack.as_mut(),
                on_open_syn.as_mut(),
                on_open_ack.as_mut(),
                on_keepalive.as_mut(),
                on_network_msg.as_mut(),
            )
            .await?;

            if !cont {
                break;
            }
        }

        Ok(())
    }
}

impl fmt::Display for TransportMessage<'_, '_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use TransportBody::*;
        match &self.body {
            InitSyn(_) => write!(f, "InitSyn"),
            InitAck(_) => write!(f, "InitAck"),
            OpenSyn(_) => write!(f, "OpenSyn"),
            OpenAck(_) => write!(f, "OpenAck"),
            KeepAlive(_) => write!(f, "KeepAlive"),
            Frame(m) => write!(f, "Frame({:?})", m.payload),
        }
    }
}

pub(crate) mod ext {
    use crate::{
        protocol::{ZCodecError, common::extension::ZExtZ64, core::Priority},
        result::ZResult,
        zbuf::{ZBufReader, ZBufWriter},
    };

    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) struct QoSType<const ID: u8> {
        inner: u8,
    }

    impl<const ID: u8> QoSType<{ ID }> {
        pub(crate) const DEFAULT: Self = Self::new(Priority::DEFAULT);

        pub(crate) const fn new(priority: Priority) -> Self {
            Self {
                inner: priority as u8,
            }
        }

        pub(crate) fn encode(
            &self,
            more: bool,
            writer: &mut ZBufWriter<'_>,
        ) -> ZResult<(), ZCodecError> {
            let ext: ZExtZ64<{ ID }> = (*self).into();
            ext.encode(more, writer)
        }

        pub(crate) fn decode(
            header: u8,
            reader: &mut ZBufReader<'_>,
        ) -> ZResult<(Self, bool), ZCodecError> {
            let (ext, more) = ZExtZ64::<{ ID }>::decode(header, reader)?;
            Ok((ext.into(), more))
        }

        #[cfg(test)]
        pub(crate) fn rand() -> Self {
            use rand::Rng;
            let mut rng = rand::thread_rng();

            let inner: u8 = rng.r#gen();
            Self { inner }
        }
    }

    impl<const ID: u8> Default for QoSType<{ ID }> {
        fn default() -> Self {
            Self::DEFAULT
        }
    }

    impl<const ID: u8> From<ZExtZ64<{ ID }>> for QoSType<{ ID }> {
        fn from(ext: ZExtZ64<{ ID }>) -> Self {
            Self {
                inner: ext.value as u8,
            }
        }
    }

    impl<const ID: u8> From<QoSType<{ ID }>> for ZExtZ64<{ ID }> {
        fn from(ext: QoSType<{ ID }>) -> Self {
            ZExtZ64::new(ext.inner as u64)
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub(crate) struct PatchType<const ID: u8>(u8);

    impl<const ID: u8> PatchType<ID> {
        pub(crate) const NONE: Self = Self(0);
        pub(crate) const CURRENT: Self = Self(1);

        pub(crate) fn encode(
            &self,
            more: bool,
            writer: &mut ZBufWriter<'_>,
        ) -> ZResult<(), ZCodecError> {
            let ext: ZExtZ64<{ ID }> = (*self).into();
            ext.encode(more, writer)
        }

        pub(crate) fn decode(
            header: u8,
            reader: &mut ZBufReader<'_>,
        ) -> ZResult<(Self, bool), ZCodecError> {
            let (ext, more) = ZExtZ64::<{ ID }>::decode(header, reader)?;
            Ok((ext.into(), more))
        }

        #[cfg(test)]
        pub(crate) fn rand() -> Self {
            use rand::Rng;
            Self(rand::thread_rng().r#gen())
        }
    }

    impl<const ID: u8> From<ZExtZ64<ID>> for PatchType<ID> {
        fn from(ext: ZExtZ64<ID>) -> Self {
            Self(ext.value as u8)
        }
    }

    impl<const ID: u8> From<PatchType<ID>> for ZExtZ64<ID> {
        fn from(ext: PatchType<ID>) -> Self {
            ZExtZ64::new(ext.0 as u64)
        }
    }
}
