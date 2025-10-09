pub mod frame;
pub mod init;
pub mod keepalive;
pub mod open;

use core::fmt;

#[cfg(test)]
use heapless::Vec;

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

pub type BatchSize = u16;
pub type AtomicBatchSize = core::sync::atomic::AtomicU16;

pub mod batch_size {
    use super::BatchSize;

    pub const UNICAST: BatchSize = BatchSize::MAX;
    pub const MULTICAST: BatchSize = 8_192;
}

pub mod id {
    pub const OAM: u8 = 0x00;
    pub const INIT: u8 = 0x01;
    pub const OPEN: u8 = 0x02;
    pub const CLOSE: u8 = 0x03;
    pub const KEEP_ALIVE: u8 = 0x04;
    pub const FRAME: u8 = 0x05;
    pub const FRAGMENT: u8 = 0x06;
    pub const JOIN: u8 = 0x07;
}

pub type TransportSn = u32;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct PrioritySn {
    pub reliable: TransportSn,
    pub best_effort: TransportSn,
}

impl PrioritySn {
    pub const DEFAULT: Self = Self {
        reliable: TransportSn::MIN,
        best_effort: TransportSn::MIN,
    };

    #[cfg(test)]
    pub fn rand() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        Self {
            reliable: rng.r#gen(),
            best_effort: rng.r#gen(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum TransportBody<'a, 'b> {
    InitSyn(InitSyn<'a>),
    InitAck(InitAck<'a>),
    OpenSyn(OpenSyn<'a>),
    OpenAck(OpenAck<'a>),
    KeepAlive(KeepAlive),
    Frame(Frame<'a, 'b>),
}

#[derive(Debug, PartialEq, Eq)]
pub struct TransportMessage<'a, 'b> {
    pub body: TransportBody<'a, 'b>,
}

impl<'a, 'b> TransportMessage<'a, 'b> {
    pub fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
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

    pub fn decode_single(
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

    pub fn decode_batch(
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

    pub async fn decode_single_async(
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

    pub async fn decode_batch_async(
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

    #[cfg(test)]
    pub fn rand(zbuf: &mut ZBufWriter<'a>, vec: &'b mut Vec<NetworkMessage<'a>, 16>) -> Self {
        use rand::Rng;

        let mut rng = rand::thread_rng();

        let body = match rng.gen_range(0..10) {
            0 => TransportBody::InitSyn(InitSyn::rand(zbuf)),
            1 => TransportBody::InitAck(InitAck::rand(zbuf)),
            2 => TransportBody::OpenSyn(OpenSyn::rand(zbuf)),
            3 => TransportBody::OpenAck(OpenAck::rand(zbuf)),
            5 => TransportBody::KeepAlive(KeepAlive::rand()),
            6 => TransportBody::Frame(Frame::rand(zbuf, vec)),
            _ => unreachable!(),
        };

        Self { body }
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

pub mod ext {
    use crate::{
        protocol::{ZCodecError, common::extension::ZExtZ64, core::Priority},
        result::ZResult,
        zbuf::{ZBufReader, ZBufWriter},
    };

    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct QoSType<const ID: u8> {
        inner: u8,
    }

    impl<const ID: u8> QoSType<{ ID }> {
        const P_MASK: u8 = 0b00000111;
        pub const DEFAULT: Self = Self::new(Priority::DEFAULT);

        pub const fn new(priority: Priority) -> Self {
            Self {
                inner: priority as u8,
            }
        }

        pub const fn priority(&self) -> Priority {
            unsafe { core::mem::transmute(self.inner & Self::P_MASK) }
        }

        pub fn encode(&self, more: bool, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
            let ext: ZExtZ64<{ ID }> = (*self).into();
            ext.encode(more, writer)
        }

        pub fn decode(
            header: u8,
            reader: &mut ZBufReader<'_>,
        ) -> ZResult<(Self, bool), ZCodecError> {
            let (ext, more) = ZExtZ64::<{ ID }>::decode(header, reader)?;
            Ok((ext.into(), more))
        }

        #[cfg(test)]
        pub fn rand() -> Self {
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
    pub struct PatchType<const ID: u8>(u8);

    impl<const ID: u8> PatchType<ID> {
        pub const NONE: Self = Self(0);
        pub const CURRENT: Self = Self(1);

        pub fn new(int: u8) -> Self {
            Self(int)
        }

        pub fn raw(self) -> u8 {
            self.0
        }

        pub fn has_fragmentation_markers(&self) -> bool {
            self.0 >= 1
        }

        pub fn encode(&self, more: bool, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
            let ext: ZExtZ64<{ ID }> = (*self).into();
            ext.encode(more, writer)
        }

        pub fn decode(
            header: u8,
            reader: &mut ZBufReader<'_>,
        ) -> ZResult<(Self, bool), ZCodecError> {
            let (ext, more) = ZExtZ64::<{ ID }>::decode(header, reader)?;
            Ok((ext.into(), more))
        }

        #[cfg(test)]
        pub fn rand() -> Self {
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
