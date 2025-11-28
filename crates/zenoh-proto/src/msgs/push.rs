use crate::{exts::*, msgs::*, *};

#[derive(ZEnum, Debug, PartialEq)]
pub enum PushBody<'a> {
    Put(Put<'a>),
}

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|M|N|ID:5=0x1d")]
pub struct Push<'a> {
    #[zenoh(flatten, shift = 5)]
    pub wire_expr: WireExpr<'a>,

    #[zenoh(ext = 0x1, default = QoS::DEFAULT)]
    pub qos: QoS,
    #[zenoh(ext = 0x2)]
    pub timestamp: Option<Timestamp>,
    #[zenoh(ext = 0x3, default = NodeId::DEFAULT, mandatory)]
    pub nodeid: NodeId,

    pub payload: PushBody<'a>,
}
