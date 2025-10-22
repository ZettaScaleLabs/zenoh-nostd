pub(crate) mod extension;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceInfo {
    pub(crate) id: crate::protocol::core::EntityGlobalIdProto,
    pub(crate) sn: u32,
}

crate::zext!(SourceInfo, crate::protocol::ext::ZExtKind::ZBuf);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Put<'a> {
    // ---------- Body for Put message ----------
    pub(crate) timestamp: Option<uhlc::Timestamp>,
    pub(crate) encoding: crate::protocol::core::encoding::Encoding<'a>,

    pub(crate) ext_sinfo: Option<SourceInfo>,

    pub(crate) payload: crate::zbuf::ZBuf<'a>,
    // ----------------------------------------
}

crate::zext!(impl<'a> SourceInfo, Put<'a>, 0x1, false);
