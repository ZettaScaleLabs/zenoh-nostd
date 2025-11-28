use crate::{exts::*, *};

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|E|T|ID:5=0x1")]
pub struct Put<'a> {
    #[zenoh(presence = header(T))]
    pub timestamp: Option<Timestamp>,
    #[zenoh(presence = header(E), default = Encoding::DEFAULT)]
    pub encoding: Encoding<'a>,

    #[zenoh(ext = 0x1)]
    pub sinfo: Option<SourceInfo>,
    #[zenoh(ext = 0x3)]
    pub attachment: Option<Attachment<'a>>,

    #[zenoh(size = prefixed)]
    pub payload: &'a [u8],
}
