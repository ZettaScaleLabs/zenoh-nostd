use crate::{exts::*, *};

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|_|I|ID:5=0x1e")]
pub struct Declare<'a> {
    #[zenoh(presence = header(I))]
    pub id: Option<u32>,

    #[zenoh(ext = 0x1, default = QoS::DEFAULT)]
    pub qos: QoS,
    #[zenoh(ext = 0x2)]
    pub timestamp: Option<Timestamp>,
    #[zenoh(ext = 0x3, default = NodeId::DEFAULT, mandatory)]
    pub nodeid: NodeId,

    pub body: DeclareBody<'a>,
}

#[derive(ZEnum, Debug, PartialEq)]
pub enum DeclareBody<'a> {
    DeclareKeyExpr(DeclareKeyExpr<'a>),
    UndeclareKeyExpr(UndeclareKeyExpr),
    DeclareSubscriber(DeclareSubscriber<'a>),
    UndeclareSubscriber(UndeclareSubscriber<'a>),
    DeclareQueryable(DeclareQueryable<'a>),
    UndeclareQueryable(UndeclareQueryable<'a>),
    DeclareToken(DeclareToken<'a>),
    UndeclareToken(UndeclareToken<'a>),
    DeclareFinal(DeclareFinal),
}

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "_|M|N|ID:5=0x00")]
pub struct DeclareKeyExpr<'a> {
    pub id: u16,
    #[zenoh(flatten, shift = 5)]
    pub wire_expr: WireExpr<'a>,
}

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "_:3|ID:5=0x01")]
pub struct UndeclareKeyExpr {
    pub id: u16,
}

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "_|M|N|ID:5=0x02")]
pub struct DeclareSubscriber<'a> {
    pub id: u32,
    #[zenoh(flatten, shift = 5)]
    pub wire_expr: WireExpr<'a>,
}

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|_:2|ID:5=0x03")]
pub struct UndeclareSubscriber<'a> {
    pub id: u32,
    #[zenoh(ext = 0x0f)]
    pub wire_expr: Option<WireExpr<'a>>,
}

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|M|N|ID:5=0x04")]
pub struct DeclareQueryable<'a> {
    pub id: u32,
    #[zenoh(flatten, shift = 5)]
    pub wire_expr: WireExpr<'a>,

    #[zenoh(ext = 0x01, default = QueryableInfo::DEFAULT)]
    pub qinfo: QueryableInfo,
}

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|_:2|ID:5=0x05")]
pub struct UndeclareQueryable<'a> {
    pub id: u32,
    #[zenoh(ext = 0x0f)]
    pub wire_expr: Option<WireExpr<'a>>,
}

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|M|N|ID:5=0x06")]
pub struct DeclareToken<'a> {
    pub id: u32,
    #[zenoh(flatten, shift = 5)]
    pub wire_expr: WireExpr<'a>,
}

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|_:2|ID:5=0x07")]
pub struct UndeclareToken<'a> {
    pub id: u32,
    #[zenoh(ext = 0x0f)]
    pub wire_expr: Option<WireExpr<'a>>,
}

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|_:2|ID:5=0x1A")]
pub struct DeclareFinal {}
