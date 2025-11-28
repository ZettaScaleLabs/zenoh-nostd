use crate::*;

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "_:3|ID:5=0x04")]
pub struct KeepAlive;
