use core::fmt;

use crate::{
    core::{wire_expr::WireExpr, CongestionControl, Priority, Reliability},
    network::{
        declare::{Declare, DeclareBody},
        interest::Interest,
        push::Push,
        request::Request,
        response::{Response, ResponseFinal},
    },
};

pub mod declare;
pub mod interest;
pub mod push;
pub mod request;
pub mod response;

pub mod id {
    // WARNING: it's crucial that these IDs do NOT collide with the IDs
    //          defined in `crate::transport::id`.
    pub const OAM: u8 = 0x1f;
    pub const DECLARE: u8 = 0x1e;
    pub const PUSH: u8 = 0x1d;
    pub const REQUEST: u8 = 0x1c;
    pub const RESPONSE: u8 = 0x1b;
    pub const RESPONSE_FINAL: u8 = 0x1a;
    pub const INTEREST: u8 = 0x19;
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Mapping {
    #[default]
    Receiver = 0,
    Sender = 1,
}

impl Mapping {
    pub const DEFAULT: Self = Self::Receiver;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkBody<'a> {
    Push(Push<'a>),
    Request(Request<'a>),
    Response(Response<'a>),
    ResponseFinal(ResponseFinal),
    Interest(Interest<'a>),
    Declare(Declare<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkMessage<'a> {
    pub body: NetworkBody<'a>,
    pub reliability: Reliability,
}

impl<'a> NetworkMessage<'a> {
    pub fn body(&self) -> &'_ NetworkBody<'_> {
        &self.body
    }

    pub fn reliability(&self) -> Reliability {
        self.reliability
    }
    #[inline]
    pub fn is_reliable(&self) -> bool {
        self.reliability() == Reliability::Reliable
    }

    #[inline]
    pub fn is_express(&self) -> bool {
        match self.body() {
            NetworkBody::Push(msg) => msg.ext_qos.is_express(),
            NetworkBody::Request(msg) => msg.ext_qos.is_express(),
            NetworkBody::Response(msg) => msg.ext_qos.is_express(),
            NetworkBody::ResponseFinal(msg) => msg.ext_qos.is_express(),
            NetworkBody::Interest(msg) => msg.ext_qos.is_express(),
            NetworkBody::Declare(msg) => msg.ext_qos.is_express(),
        }
    }

    #[inline]
    pub fn congestion_control(&self) -> CongestionControl {
        match self.body() {
            NetworkBody::Push(msg) => msg.ext_qos.get_congestion_control(),
            NetworkBody::Request(msg) => msg.ext_qos.get_congestion_control(),
            NetworkBody::Response(msg) => msg.ext_qos.get_congestion_control(),
            NetworkBody::ResponseFinal(msg) => msg.ext_qos.get_congestion_control(),
            NetworkBody::Interest(msg) => msg.ext_qos.get_congestion_control(),
            NetworkBody::Declare(msg) => msg.ext_qos.get_congestion_control(),
        }
    }

    #[inline]
    pub fn is_droppable(&self) -> bool {
        !self.is_reliable() || self.congestion_control() == CongestionControl::Drop
    }

    #[inline]
    pub fn priority(&self) -> Priority {
        match self.body() {
            NetworkBody::Push(msg) => msg.ext_qos.get_priority(),
            NetworkBody::Request(msg) => msg.ext_qos.get_priority(),
            NetworkBody::Response(msg) => msg.ext_qos.get_priority(),
            NetworkBody::ResponseFinal(msg) => msg.ext_qos.get_priority(),
            NetworkBody::Interest(msg) => msg.ext_qos.get_priority(),
            NetworkBody::Declare(msg) => msg.ext_qos.get_priority(),
        }
    }

    #[inline]
    pub fn wire_expr(&self) -> Option<&'_ WireExpr<'_>> {
        match &self.body() {
            NetworkBody::Push(m) => Some(&m.wire_expr),
            NetworkBody::Request(m) => Some(&m.wire_expr),
            NetworkBody::Response(m) => Some(&m.wire_expr),
            NetworkBody::ResponseFinal(_) => None,
            NetworkBody::Interest(m) => m.wire_expr.as_ref(),
            NetworkBody::Declare(m) => match &m.body {
                DeclareBody::DeclareKeyExpr(m) => Some(&m.wire_expr),
                DeclareBody::UndeclareKeyExpr(_) => None,
                DeclareBody::DeclareSubscriber(m) => Some(&m.wire_expr),
                DeclareBody::UndeclareSubscriber(m) => Some(&m.ext_wire_expr.wire_expr),
                DeclareBody::DeclareQueryable(m) => Some(&m.wire_expr),
                DeclareBody::UndeclareQueryable(m) => Some(&m.ext_wire_expr.wire_expr),
                DeclareBody::DeclareToken(m) => Some(&m.wire_expr),
                DeclareBody::UndeclareToken(m) => Some(&m.ext_wire_expr.wire_expr),
                DeclareBody::DeclareFinal(_) => None,
            },
        }
    }
}

impl fmt::Display for NetworkMessage<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.body {
            NetworkBody::Push(_) => write!(f, "Push"),
            NetworkBody::Request(_) => write!(f, "Request"),
            NetworkBody::Response(_) => write!(f, "Response"),
            NetworkBody::ResponseFinal(_) => write!(f, "ResponseFinal"),
            NetworkBody::Interest(_) => write!(f, "Interest"),
            NetworkBody::Declare(_) => write!(f, "Declare"),
        }
    }
}

impl<'a> From<NetworkBody<'a>> for NetworkMessage<'a> {
    #[inline]
    fn from(body: NetworkBody<'a>) -> Self {
        Self {
            body,
            reliability: Reliability::DEFAULT,
        }
    }
}

// Extensions
pub mod ext {
    use core::fmt;

    use crate::{
        common::{extension::ZExtZ64, imsg},
        core::{CongestionControl, EntityId, Priority, ZenohIdProto},
    };

    /// ```text
    ///  7 6 5 4 3 2 1 0
    /// +-+-+-+-+-+-+-+-+
    /// |Z|0_1|    ID   |
    /// +-+-+-+---------+
    /// %0|r|F|E|D|prio %
    /// +---------------+
    ///
    /// - prio: Priority class
    /// - D:    Don't drop. Don't drop the message for congestion control.
    /// - E:    Express. Don't batch this message.
    /// - F:    Don't drop the first message for congestion control.
    /// - r:  Reserved
    /// ```
    #[repr(transparent)]
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct QoSType<const ID: u8> {
        inner: u8,
    }

    impl<const ID: u8> QoSType<{ ID }> {
        const P_MASK: u8 = 0b00000111;
        const D_FLAG: u8 = 0b00001000;
        const E_FLAG: u8 = 0b00010000;
        const F_FLAG: u8 = 0b00100000;

        pub const DEFAULT: Self = Self::new(Priority::DEFAULT, CongestionControl::DEFAULT, false);

        pub const DECLARE: Self =
            Self::new(Priority::Control, CongestionControl::DEFAULT_DECLARE, false);
        pub const PUSH: Self = Self::new(Priority::DEFAULT, CongestionControl::DEFAULT_PUSH, false);
        pub const REQUEST: Self =
            Self::new(Priority::DEFAULT, CongestionControl::DEFAULT_REQUEST, false);
        pub const RESPONSE: Self = Self::new(
            Priority::DEFAULT,
            CongestionControl::DEFAULT_RESPONSE,
            false,
        );
        pub const RESPONSE_FINAL: Self = Self::new(
            Priority::DEFAULT,
            CongestionControl::DEFAULT_RESPONSE,
            false,
        );
        pub const OAM: Self = Self::new(Priority::Control, CongestionControl::DEFAULT_OAM, false);

        pub const fn new(
            priority: Priority,
            congestion_control: CongestionControl,
            is_express: bool,
        ) -> Self {
            let mut inner = priority as u8;
            match congestion_control {
                CongestionControl::Block => inner |= Self::D_FLAG,
                _ => {}
            }
            if is_express {
                inner |= Self::E_FLAG;
            }
            Self { inner }
        }

        pub fn set_priority(&mut self, priority: Priority) {
            self.inner = imsg::set_bitfield(self.inner, priority as u8, Self::P_MASK);
        }

        pub const fn get_priority(&self) -> Priority {
            unsafe { core::mem::transmute(self.inner & Self::P_MASK) }
        }

        pub fn set_congestion_control(&mut self, cctrl: CongestionControl) {
            match cctrl {
                CongestionControl::Block => {
                    self.inner = imsg::set_flag(self.inner, Self::D_FLAG);
                    self.inner = imsg::unset_flag(self.inner, Self::F_FLAG);
                }
                CongestionControl::Drop => {
                    self.inner = imsg::unset_flag(self.inner, Self::D_FLAG);
                    self.inner = imsg::unset_flag(self.inner, Self::F_FLAG);
                }
            }
        }

        pub const fn get_congestion_control(&self) -> CongestionControl {
            match (
                imsg::has_flag(self.inner, Self::D_FLAG),
                imsg::has_flag(self.inner, Self::F_FLAG),
            ) {
                (false, false) => CongestionControl::Drop,
                (false, true) => CongestionControl::Drop,
                (true, _) => CongestionControl::Block,
            }
        }

        pub fn set_is_express(&mut self, is_express: bool) {
            match is_express {
                true => self.inner = imsg::set_flag(self.inner, Self::E_FLAG),
                false => self.inner = imsg::unset_flag(self.inner, Self::E_FLAG),
            }
        }

        pub const fn is_express(&self) -> bool {
            imsg::has_flag(self.inner, Self::E_FLAG)
        }
    }

    impl<const ID: u8> Default for QoSType<{ ID }> {
        fn default() -> Self {
            Self::new(Priority::DEFAULT, CongestionControl::DEFAULT, false)
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

    impl<const ID: u8> fmt::Debug for QoSType<{ ID }> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.debug_struct("QoS")
                .field("priority", &self.get_priority())
                .field("congestion", &self.get_congestion_control())
                .field("express", &self.is_express())
                .finish()
        }
    }

    /// ```text
    ///  7 6 5 4 3 2 1 0
    /// +-+-+-+-+-+-+-+-+
    /// |Z|1_0|    ID   |
    /// +-+-+-+---------+
    /// ~ ts: <u8;z16>  ~
    /// +---------------+
    /// ```
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct TimestampType<const ID: u8> {
        pub timestamp: uhlc::Timestamp,
    }

    /// ```text
    ///  7 6 5 4 3 2 1 0
    /// +-+-+-+-+-+-+-+-+
    /// |Z|0_1|    ID   |
    /// +-+-+-+---------+
    /// %    node_id    %
    /// +---------------+
    /// ```
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct NodeIdType<const ID: u8> {
        pub node_id: u16,
    }

    impl<const ID: u8> NodeIdType<{ ID }> {
        // node_id == 0 means the message has been generated by the node itself
        pub const DEFAULT: Self = Self { node_id: 0 };
    }

    impl<const ID: u8> Default for NodeIdType<{ ID }> {
        fn default() -> Self {
            Self::DEFAULT
        }
    }

    impl<const ID: u8> From<ZExtZ64<{ ID }>> for NodeIdType<{ ID }> {
        fn from(ext: ZExtZ64<{ ID }>) -> Self {
            Self {
                node_id: ext.value as u16,
            }
        }
    }

    impl<const ID: u8> From<NodeIdType<{ ID }>> for ZExtZ64<{ ID }> {
        fn from(ext: NodeIdType<{ ID }>) -> Self {
            ZExtZ64::new(ext.node_id as u64)
        }
    }

    /// ```text
    ///  7 6 5 4 3 2 1 0
    /// +-+-+-+-+-+-+-+-+
    /// |zid_len|X|X|X|X|
    /// +-------+-+-+---+
    /// ~      zid      ~
    /// +---------------+
    /// %      eid      %
    /// +---------------+
    /// ```
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct EntityGlobalIdType<const ID: u8> {
        pub zid: ZenohIdProto,
        pub eid: EntityId,
    }
}
