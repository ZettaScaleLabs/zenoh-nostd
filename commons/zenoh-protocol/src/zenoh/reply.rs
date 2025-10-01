use heapless::Vec;

use crate::{
    common::extension::ZExtUnknown,
    zenoh::{query::ConsolidationMode, PushBody},
};

/// # Reply message
///
/// ```text
/// Flags:
/// - C: Consolidation  if C==1 then consolidation is present
/// - X: Reserved
/// - Z: Extension      If Z==1 then at least one extension is present
///
///   7 6 5 4 3 2 1 0
///  +-+-+-+-+-+-+-+-+
///  |Z|X|C|  REPLY  |
///  +-+-+-+---------+
///  % consolidation %  if C==1
///  +---------------+
///  ~  [repl_exts]  ~  if Z==1
///  +---------------+
///  ~   ReplyBody   ~  -- Payload
///  +---------------+
/// ```
pub mod flag {
    pub const C: u8 = 1 << 5; // 0x20 Consolidation if C==1 then consolidation is present
                              // pub const X: u8 = 1 << 6; // 0x40 Reserved
    pub const Z: u8 = 1 << 7; // 0x80 Extensions    if Z==1 then an extension will follow
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reply<'a, const MAX_EXT_UNKNOWN: usize> {
    pub consolidation: ConsolidationMode,
    pub ext_unknown: Vec<ZExtUnknown<'a>, MAX_EXT_UNKNOWN>,
    pub payload: ReplyBody<'a, MAX_EXT_UNKNOWN>,
}

pub type ReplyBody<'a, const MAX_EXT_UNKNOWN: usize> = PushBody<'a, MAX_EXT_UNKNOWN>;
