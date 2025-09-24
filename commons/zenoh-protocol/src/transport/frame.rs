use zenoh_buffers::zslice::ZSlice;

use crate::{core::Reliability, transport::TransportSn};

pub mod flag {
    pub const R: u8 = 1 << 5; // 0x20 Reliable      if R==1 then the frame is reliable
                              // pub const X: u8 = 1 << 6; // 0x40       Reserved
    pub const Z: u8 = 1 << 7; // 0x80 Extensions    if Z==1 then an extension will follow
}

/// # Frame message
///
/// The [`Frame`] message is used to transmit one ore more complete serialized
/// [`crate::network::NetworkMessage`]. I.e., the total length of the
/// serialized [`crate::network::NetworkMessage`] (s) MUST be smaller
/// than the maximum batch size (i.e. 2^16-1) and the link MTU.
/// The [`Frame`] message is used as means to aggregate multiple
/// [`crate::network::NetworkMessage`] in a single atomic message that
/// goes on the wire. By doing so, many small messages can be batched together and
/// share common information like the sequence number.
///
/// The [`Frame`] message flow is the following:
///
/// ```text
///     A                   B
///     |      FRAME        |
///     |------------------>|
///     |                   |
/// ```
///
/// The [`Frame`] message structure is defined as follows:
///
/// ```text
/// Flags:
/// - R: Reliable       If R==1 it concerns the reliable channel, else the best-effort channel
/// - X: Reserved
/// - Z: Extensions     If Z==1 then zenoh extensions will follow.
///
///  7 6 5 4 3 2 1 0
/// +-+-+-+-+-+-+-+-+
/// |Z|X|R|  FRAME  |
/// +-+-+-+---------+
/// %    seq num    %
/// +---------------+
/// ~  [FrameExts]  ~ if Flag(Z)==1
/// +---------------+
/// ~  [NetworkMsg] ~
/// +---------------+
/// ```
///
/// NOTE: 16 bits (2 bytes) may be prepended to the serialized message indicating the total length
///       in bytes of the message, resulting in the maximum length of a message being 65535 bytes.
///       This is necessary in those stream-oriented transports (e.g., TCP) that do not preserve
///       the boundary of the serialized messages. The length is encoded as little-endian.
///       In any case, the length of a message must not exceed 65535 bytes.
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub reliability: Reliability,
    pub sn: TransportSn,
    pub ext_qos: ext::QoSType,
    pub payload: ZSlice,
}

// Extensions
pub mod ext {
    use crate::{common::ZExtZ64, zextz64};

    pub type QoS = zextz64!(0x1, true);
    pub type QoSType = crate::transport::ext::QoSType<{ QoS::ID }>;
}

// FrameHeader
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct FrameHeader {
    pub reliability: Reliability,
    pub sn: TransportSn,
    pub ext_qos: ext::QoSType,
}
