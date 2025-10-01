//! # Open message
//!
//! After having successfully complete the [`super::InitSyn`]-[`super::InitAck`] message exchange,
//! the OPEN message is sent on a link to finalize the initialization of the link and
//! associated transport with a zenoh node.
//! For convenience, we call [`OpenSyn`] and [`OpenAck`] an OPEN message with the A flag
//! is set to 0 and 1, respectively.
//!
//! The [`OpenSyn`]/[`OpenAck`] message flow is the following:
//!
//! ```text
//!     A                   B
//!     |      OPEN SYN     |
//!     |------------------>|
//!     |                   |
//!     |      OPEN ACK     |
//!     |<------------------|
//!     |                   |
//! ```
//!
//! ```text
//! Flags:
//! - A: Ack            If A==0 then the message is an OpenSyn else it is an OpenAck
//! - T: Lease period   if T==1 then the lease period is in seconds else in milliseconds
//! - Z: Extensions     If Z==1 then zenoh extensions will follow.
//!
//!  7 6 5 4 3 2 1 0
//! +-+-+-+-+-+-+-+-+
//! |Z|T|A|   OPEN  |
//! +-+-+-+---------+
//! %     lease     % -- Lease period of the sender of the OPEN message
//! +---------------+
//! %  initial_sn   % -- Initial SN proposed by the sender of the OPEN(*)
//! +---------------+
//! ~    <u8;z16>   ~ if Flag(A)==0 (**) -- Cookie
//! +---------------+
//! ~   [OpenExts]  ~ if Flag(Z)==1
//! +---------------+
//!
//! (*)     The initial sequence number MUST be compatible with the sequence number resolution agreed in the
//!         [`super::InitSyn`]-[`super::InitAck`] message exchange
//! (**)    The cookie MUST be the same received in the [`super::InitAck`]from the corresponding zenoh node
//! ```
//!
//! NOTE: 16 bits (2 bytes) may be prepended to the serialized message indicating the total length
//!       in bytes of the message, resulting in the maximum length of a message being 65535 bytes.
//!       This is necessary in those stream-oriented transports (e.g., TCP) that do not preserve
//!       the boundary of the serialized messages. The length is encoded as little-endian.
//!       In any case, the length of a message must not exceed 65535 bytes.

use core::time::Duration;

use zenoh_buffer::ZBuf;

use crate::transport::TransportSn;

pub mod flag {
    pub const A: u8 = 1 << 5; // 0x20 Ack           if A==0 then the message is an InitSyn else it is an InitAck
    pub const T: u8 = 1 << 6; // 0x40 Lease period  if T==1 then the lease period is in seconds else in milliseconds
    pub const Z: u8 = 1 << 7; // 0x80 Extensions    if Z==1 then an extension will follow
}

/// # OpenSyn message
///
/// ```text
///  7 6 5 4 3 2 1 0
/// +-+-+-+-+-+-+-+-+
/// ~   challenge   ~
/// +---------------+
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenSyn<'a> {
    pub lease: Duration,
    pub initial_sn: TransportSn,
    pub cookie: ZBuf<'a>,
    pub ext_qos: Option<ext::QoS>,
    pub ext_auth: Option<ext::Auth<'a>>,
    pub ext_mlink: Option<ext::MultiLinkSyn<'a>>,
    pub ext_lowlatency: Option<ext::LowLatency>,
    pub ext_compression: Option<ext::Compression>,
}

// Extensions
pub mod ext {
    use crate::{zextunit, zextzbuf};

    /// # QoS extension
    /// Used to negotiate the use of QoS
    pub type QoS = zextunit!(0x1, false);

    /// # Auth extension
    /// Used as challenge for probing authentication rights
    pub type Auth<'a> = zextzbuf!('a, 0x3, false);

    /// # Multilink extension
    /// Used as challenge for probing multilink capabilities
    pub type MultiLinkSyn<'a> = zextzbuf!('a, 0x4, false);
    pub type MultiLinkAck = zextunit!(0x4, false);

    /// # LowLatency extension
    /// Used to negotiate the use of lowlatency transport
    pub type LowLatency = zextunit!(0x5, false);

    /// # Compression extension
    /// Used to negotiate the use of compression on the link
    pub type Compression = zextunit!(0x6, false);
}

/// # OpenAck message
///
/// ```text
///  7 6 5 4 3 2 1 0
/// +-+-+-+-+-+-+-+-+
/// ~      ack      ~
/// +---------------+
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenAck<'a> {
    pub lease: Duration,
    pub initial_sn: TransportSn,
    pub ext_qos: Option<ext::QoS>,
    pub ext_auth: Option<ext::Auth<'a>>,
    pub ext_mlink: Option<ext::MultiLinkAck>,
    pub ext_lowlatency: Option<ext::LowLatency>,
    pub ext_compression: Option<ext::Compression>,
}
