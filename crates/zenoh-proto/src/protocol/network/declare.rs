use uhlc::Timestamp;

#[cfg(test)]
use rand::Rng;
use zenoh_derive::ZEnum;

use crate::{
    WireExpr, ZStruct,
    network::{NodeId, QoS, QueryableInfo},
};

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

impl<'a> Declare<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut crate::ZWriter<'a>) -> Self {
        let id = if rand::thread_rng().gen_bool(0.5) {
            Some(rand::thread_rng().r#gen())
        } else {
            None
        };

        let qos = if rand::thread_rng().gen_bool(0.5) {
            QoS::rand(w)
        } else {
            QoS::DEFAULT
        };

        let timestamp = rand::thread_rng().gen_bool(0.5).then_some({
            use crate::protocol::core::ZenohIdProto;

            let time = uhlc::NTP64(rand::thread_rng().r#gen());
            let id = uhlc::ID::try_from(ZenohIdProto::default().as_le_bytes()).unwrap();
            Timestamp::new(time, id)
        });

        let nodeid = if rand::thread_rng().gen_bool(0.5) {
            NodeId::rand(w)
        } else {
            NodeId::DEFAULT
        };

        let body = DeclareBody::DeclareKeyExpr(DeclareKeyExpr::rand(w));

        Self {
            id,
            qos,
            timestamp,
            nodeid,
            body,
        }
    }
}

impl<'a> DeclareKeyExpr<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut crate::ZWriter<'a>) -> Self {
        let id = rand::thread_rng().r#gen();
        let wire_expr = WireExpr::rand(w);
        Self { id, wire_expr }
    }
}

impl UndeclareKeyExpr {
    #[cfg(test)]
    pub(crate) fn rand(_: &mut crate::ZWriter) -> Self {
        let id = rand::thread_rng().r#gen();
        Self { id }
    }
}

impl<'a> DeclareSubscriber<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut crate::ZWriter<'a>) -> Self {
        let id = rand::thread_rng().r#gen();
        let wire_expr = WireExpr::rand(w);
        Self { id, wire_expr }
    }
}

impl<'a> UndeclareSubscriber<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut crate::ZWriter<'a>) -> Self {
        let id = rand::thread_rng().r#gen();
        let wire_expr = if rand::thread_rng().gen_bool(0.5) {
            Some(WireExpr::rand(w))
        } else {
            None
        };
        Self { id, wire_expr }
    }
}

impl<'a> DeclareQueryable<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut crate::ZWriter<'a>) -> Self {
        let id = rand::thread_rng().r#gen();
        let wire_expr = WireExpr::rand(w);
        let qinfo = if rand::thread_rng().gen_bool(0.5) {
            QueryableInfo::rand(w)
        } else {
            QueryableInfo::DEFAULT
        };
        Self {
            id,
            wire_expr,
            qinfo,
        }
    }
}

impl<'a> UndeclareQueryable<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut crate::ZWriter<'a>) -> Self {
        let id = rand::thread_rng().r#gen();
        let wire_expr = if rand::thread_rng().gen_bool(0.5) {
            Some(WireExpr::rand(w))
        } else {
            None
        };
        Self { id, wire_expr }
    }
}

impl<'a> DeclareToken<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut crate::ZWriter<'a>) -> Self {
        let id = rand::thread_rng().r#gen();
        let wire_expr = WireExpr::rand(w);
        Self { id, wire_expr }
    }
}

impl<'a> UndeclareToken<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut crate::ZWriter<'a>) -> Self {
        let id = rand::thread_rng().r#gen();
        let wire_expr = if rand::thread_rng().gen_bool(0.5) {
            Some(WireExpr::rand(w))
        } else {
            None
        };
        Self { id, wire_expr }
    }
}

impl DeclareFinal {
    #[cfg(test)]
    pub(crate) fn rand(_: &mut crate::ZWriter) -> Self {
        Self {}
    }
}
