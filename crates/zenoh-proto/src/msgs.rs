pub mod exts;
pub mod fields;

mod err;
mod put;
mod query;
mod reply;

mod declare;
mod interest;
mod push;
mod request;
mod response;

mod close;
mod frame;
mod init;
mod keepalive;
mod open;

pub use err::*;
pub use put::*;
pub use query::*;
pub use reply::*;

pub use declare::*;
pub use interest::*;
pub use push::*;
pub use request::*;
pub use response::*;

pub use close::*;
pub use frame::*;
pub use init::*;
pub use keepalive::*;
pub use open::*;

use zenoh_derive::ZEnum;

#[derive(ZEnum, Debug, PartialEq)]
pub enum NetworkBody<'a> {
    Push(Push<'a>),
    Request(Request<'a>),
    Response(Response<'a>),
    ResponseFinal(ResponseFinal),
    Interest(Interest<'a>),
    InterestFinal(InterestFinal),
    Declare(Declare<'a>),
}

#[derive(Debug, PartialEq)]
pub struct NetworkMessage<'a> {
    pub reliability: fields::Reliability,
    pub qos: exts::QoS,
    pub body: NetworkBody<'a>,
}

#[derive(ZEnum, Debug, PartialEq)]
pub enum TransportMessage<'a> {
    Close(Close),
    InitSyn(InitSyn<'a>),
    InitAck(InitAck<'a>),
    KeepAlive(KeepAlive),
    OpenSyn(OpenSyn<'a>),
    OpenAck(OpenAck<'a>),
}

#[derive(Debug, PartialEq)]
pub enum Message<'a> {
    Network(NetworkMessage<'a>),
    Transport(TransportMessage<'a>),
}

#[derive(Debug, PartialEq)]
pub enum MessageRef<'a> {
    Network(&'a NetworkMessage<'a>),
    Transport(&'a TransportMessage<'a>),
}

impl Message<'_> {
    pub fn as_ref(&self) -> MessageRef<'_> {
        match self {
            Message::Transport(msg) => MessageRef::Transport(msg),
            Message::Network(msg) => MessageRef::Network(msg),
        }
    }
}
