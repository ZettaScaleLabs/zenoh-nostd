use crate::{
    Reliability, ZStruct,
    network::{NetworkBatch, QoS},
};

#[cfg(test)]
use rand::Rng;

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|_|R|ID:5=0x05")]
pub struct FrameHeader {
    #[zenoh(header = R)]
    pub reliability: Reliability,
    pub sn: u32,

    #[zenoh(ext = 0x1, default = QoS::DEFAULT)]
    pub qos: QoS,
}

impl Frame<'_, '_> {
    pub const ID: u8 = FrameHeader::ID;
}

#[derive(Debug, PartialEq)]
pub struct Frame<'a, 'b> {
    pub header: FrameHeader,
    pub msgs: NetworkBatch<'a, 'b>,
}

impl Drop for Frame<'_, '_> {
    fn drop(&mut self) {
        for _ in self.msgs.by_ref() {}
    }
}

impl FrameHeader {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut crate::ZWriter) -> Self {
        let reliability = Reliability::rand(w);
        let sn = rand::thread_rng().r#gen();
        let qos = QoS::rand(w);
        Self {
            reliability,
            sn,
            qos,
        }
    }
}
