use crate::*;

#[repr(u8)]
#[derive(ZRU8, Debug, Default, PartialEq, Clone, Copy)]
pub enum Mapping {
    #[default]
    Receiver = 0,
    Sender = 1,
}

#[derive(ZExt, Debug, PartialEq)]
#[zenoh(header = "_:6|M|N|")]
pub struct WireExpr<'a> {
    pub scope: u16,

    #[zenoh(header = M)]
    pub mapping: Mapping,

    #[zenoh(presence = header(N), default = "", size = prefixed)]
    pub suffix: &'a str,
}

impl<'a> From<&'a keyexpr> for WireExpr<'a> {
    fn from(ke: &'a keyexpr) -> Self {
        Self {
            scope: 0,
            mapping: Mapping::Sender,
            suffix: ke.as_str(),
        }
    }
}
