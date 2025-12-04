use core::time::Duration;

use crate::{exts::*, *};

#[derive(ZStruct, Debug, PartialEq, Default)]
#[zenoh(header = "Z|T|A:1=0|ID:5=0x02")]
pub struct OpenSyn<'a> {
    #[zenoh(flatten, shift = 6)]
    pub lease: Duration,
    pub sn: u32,

    #[zenoh(size = prefixed)]
    pub cookie: &'a [u8],

    #[zenoh(ext = 0x1)]
    pub qos: Option<HasQoS>,
    #[zenoh(ext = 0x3)]
    pub auth: Option<Auth<'a>>,
    #[zenoh(ext = 0x4)]
    pub mlink_syn: Option<MultiLinkSyn<'a>>,
    #[zenoh(ext = 0x4)]
    pub mlink_ack: Option<HasMultiLinkAck>,
    #[zenoh(ext = 0x5)]
    pub lowlatency: Option<HasLowLatency>,
    #[zenoh(ext = 0x6)]
    pub compression: Option<HasCompression>,
}

#[derive(ZStruct, Debug, PartialEq, Default)]
#[zenoh(header = "Z|T|A:1=1|ID:5=0x02")]
pub struct OpenAck<'a> {
    #[zenoh(flatten, shift = 6)]
    pub lease: Duration,
    pub sn: u32,

    #[zenoh(ext = 0x1)]
    pub qos: Option<HasQoS>,
    #[zenoh(ext = 0x3)]
    pub auth: Option<Auth<'a>>,
    #[zenoh(ext = 0x4)]
    pub mlink_syn: Option<MultiLinkSyn<'a>>,
    #[zenoh(ext = 0x4)]
    pub mlink_ack: Option<HasMultiLinkAck>,
    #[zenoh(ext = 0x5)]
    pub lowlatency: Option<HasLowLatency>,
    #[zenoh(ext = 0x6)]
    pub compression: Option<HasCompression>,
}
