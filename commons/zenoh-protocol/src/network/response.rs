use crate::{core::wire_expr::WireExpr, network::request::RequestId, zenoh::ResponseBody};

/// # Response message
///
/// ```text
/// Flags:
/// - N: Named          If N==1 then the key expr has name/suffix
/// - M: Mapping        if M==1 then key expr mapping is the one declared by the sender, else it is the one declared by the receiver
/// - Z: Extension      If Z==1 then at least one extension is present
///
///  7 6 5 4 3 2 1 0
/// +-+-+-+-+-+-+-+-+
/// |Z|M|N| Response|
/// +-+-+-+---------+
/// ~ request_id:z32~  (*)
/// +---------------+
/// ~ key_scope:z16 ~
/// +---------------+
/// ~  key_suffix   ~  if N==1 -- <u8;z16>
/// +---------------+
/// ~  [reply_exts] ~  if Z==1
/// +---------------+
/// ~  ResponseBody ~ -- Payload
/// +---------------+
///
/// (*) The resolution of the request id is negotiated during the session establishment.
///     This implementation limits the resolution to 32bit.
/// ```
pub mod flag {
    pub const N: u8 = 1 << 5; // 0x20 Named         if N==1 then the key expr has name/suffix
    pub const M: u8 = 1 << 6; // 0x40 Mapping       if M==1 then key expr mapping is the one declared by the sender, else it is the one declared by the receiver
    pub const Z: u8 = 1 << 7; // 0x80 Extensions    if Z==1 then an extension will follow
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response<'a, const MAX_EXT_UNKNOWN: usize> {
    pub rid: RequestId,
    pub wire_expr: WireExpr<'a>,
    pub payload: ResponseBody<'a, MAX_EXT_UNKNOWN>,
    pub ext_qos: ext::QoSType,
    pub ext_tstamp: Option<ext::TimestampType>,
    pub ext_respid: Option<ext::ResponderIdType>,
}

pub mod ext {
    use crate::{zextz64, zextzbuf};
    pub type QoS = zextz64!(0x1, false);
    pub type QoSType = crate::network::ext::QoSType<{ QoS::ID }>;

    pub type Timestamp<'a> = zextzbuf!('a, 0x2, false);
    pub type TimestampType = crate::network::ext::TimestampType<{ Timestamp::ID }>;

    pub type ResponderId<'a> = zextzbuf!('a, 0x3, false);
    pub type ResponderIdType = crate::network::ext::EntityGlobalIdType<{ ResponderId::ID }>;
}

/// # ResponseFinal message
///
/// ```text
/// Flags:
/// - X: Reserved
/// - X: Reserved
/// - Z: Extension      If Z==1 then at least one extension is present
///
///  7 6 5 4 3 2 1 0
/// +-+-+-+-+-+-+-+-+
/// |Z|X|X| ResFinal|
/// +-+-+-+---------+
/// ~ request_id:z32~  (*)
/// +---------------+
/// ~  [reply_exts] ~  if Z==1
/// +---------------+
///
/// (*) The resolution of the request id is negotiated during the session establishment.
///     This implementation limits the resolution to 32bit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResponseFinal {
    pub rid: RequestId,
    pub ext_qos: ext::QoSType,
    pub ext_tstamp: Option<ext::TimestampType>,
}
