use ryu_derive::ZExt;

#[derive(ZExt, Debug, PartialEq)]
pub struct Path {
    pub patch: u64,
}
