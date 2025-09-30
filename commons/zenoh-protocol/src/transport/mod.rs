pub mod frame;
pub mod init;
pub mod keepalive;
pub mod open;

use core::fmt;

pub use frame::{Frame, FrameHeader};
pub use init::{InitAck, InitSyn};
pub use keepalive::KeepAlive;
pub use open::{OpenAck, OpenSyn};

// use crate::network::{NetworkMessage, NetworkMessageRef};

/// NOTE: 16 bits (2 bytes) may be prepended to the serialized message indicating the total length
///       in bytes of the message, resulting in the maximum length of a message being 65_535 bytes.
///       This is necessary in those stream-oriented transports (e.g., TCP) that do not preserve
///       the boundary of the serialized messages. The length is encoded as little-endian.
///       In any case, the length of a message must not exceed 65_535 bytes.
pub type BatchSize = u16;
pub type AtomicBatchSize = core::sync::atomic::AtomicU16;

pub mod batch_size {
    use super::BatchSize;

    pub const UNICAST: BatchSize = BatchSize::MAX;
    pub const MULTICAST: BatchSize = 8_192;
}

pub mod id {
    // WARNING: it's crucial that these IDs do NOT collide with the IDs
    //          defined in `crate::network::id`.
    pub const OAM: u8 = 0x00;
    pub const INIT: u8 = 0x01; // For unicast communications only
    pub const OPEN: u8 = 0x02; // For unicast communications only
    pub const CLOSE: u8 = 0x03;
    pub const KEEP_ALIVE: u8 = 0x04;
    pub const FRAME: u8 = 0x05;
    pub const FRAGMENT: u8 = 0x06;
    pub const JOIN: u8 = 0x07; // For multicast communications only
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportBody<'a> {
    InitSyn(InitSyn<'a>),
    InitAck(InitAck<'a>),
    OpenSyn(OpenSyn<'a>),
    OpenAck(OpenAck<'a>),
    KeepAlive(KeepAlive),
    Frame(Frame<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportMessage<'a> {
    pub body: TransportBody<'a>,
}

impl<'a> From<TransportBody<'a>> for TransportMessage<'a> {
    fn from(body: TransportBody<'a>) -> Self {
        Self { body }
    }
}

impl<'a> From<InitSyn<'a>> for TransportMessage<'a> {
    fn from(init_syn: InitSyn<'a>) -> Self {
        TransportBody::InitSyn(init_syn).into()
    }
}

impl<'a> From<InitAck<'a>> for TransportMessage<'a> {
    fn from(init_ack: InitAck<'a>) -> Self {
        TransportBody::InitAck(init_ack).into()
    }
}

impl<'a> From<OpenSyn<'a>> for TransportMessage<'a> {
    fn from(open_syn: OpenSyn<'a>) -> Self {
        TransportBody::OpenSyn(open_syn).into()
    }
}

impl<'a> From<OpenAck<'a>> for TransportMessage<'a> {
    fn from(open_ack: OpenAck<'a>) -> Self {
        TransportBody::OpenAck(open_ack).into()
    }
}

impl<'a> From<KeepAlive> for TransportMessage<'a> {
    fn from(keep_alive: KeepAlive) -> Self {
        TransportBody::KeepAlive(keep_alive).into()
    }
}

impl<'a> From<Frame<'a>> for TransportMessage<'a> {
    fn from(frame: Frame<'a>) -> Self {
        TransportBody::Frame(frame).into()
    }
}

impl<'a> fmt::Display for TransportMessage<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use TransportBody::*;
        match &self.body {
            InitSyn(_) => write!(f, "InitSyn"),
            InitAck(_) => write!(f, "InitAck"),
            OpenSyn(_) => write!(f, "OpenSyn"),
            OpenAck(_) => write!(f, "OpenAck"),
            KeepAlive(_) => write!(f, "KeepAlive"),
            Frame(m) => {
                write!(f, "Frame[")?;
                let mut netmsgs = m.payload.iter().peekable();
                while let Some(m) = netmsgs.next() {
                    m.fmt(f)?;
                    if netmsgs.peek().is_some() {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "]")
            }
        }
    }
}

pub mod ext {
    use crate::{common::extension::ZExtZ64, core::Priority};

    /// ```text
    ///  7 6 5 4 3 2 1 0
    /// +-+-+-+-+-+-+-+-+
    /// %0|  rsv  |prio %
    /// +---------------+
    /// - prio: Priority class
    /// ```
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
