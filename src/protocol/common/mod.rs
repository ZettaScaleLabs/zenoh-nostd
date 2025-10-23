use crate::protocol::ext::{ZExtPrimitive, encode_ext_header};

pub(crate) mod extension;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceInfo {
    pub(crate) sn: u32,
}

pub(crate) fn encode_source_info(writer: &mut crate::zbuf::) {}

crate::zext!(SourceInfo, crate::protocol::ext::ZExtKind::Z64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Put<'a> {
    // ---------- Body for Put message ----------
    pub(crate) timestamp: Option<uhlc::Timestamp>,
    pub(crate) encoding: crate::protocol::core::encoding::Encoding<'a>,

    pub(crate) ext_sinfo: Option<SourceInfo>,

    pub(crate) payload: crate::zbuf::ZBuf<'a>,
    // ----------------------------------------
}

impl<'a> Put<'a> {
    fn encode(&self, writer: &mut crate::zbuf::ZBufWriter<'_>) {
        encode_ext_header::<SourceInfo, Self>(writer, true).unwrap();
    }
}

crate::zext!(impl<'a> SourceInfo, Put<'a>, 0x1, false);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Abd<'a> {
    // ---------- Body for Put message ----------
    pub(crate) timestamp: Option<uhlc::Timestamp>,
    pub(crate) encoding: crate::protocol::core::encoding::Encoding<'a>,

    pub(crate) ext_sinfo: Option<SourceInfo>,

    pub(crate) payload: crate::zbuf::ZBuf<'a>,
    // ----------------------------------------
}

crate::zext!(impl<'a> SourceInfo, Abd<'a>, 0x1, false);
