use crate::{
    Reliability, ZCodecResult, ZEncode, ZWriter,
    network::{NetworkBody, QoS},
    transport::frame::FrameHeader,
};

pub struct Batch<'a> {
    writer: ZWriter<'a>,
    frame: Option<Reliability>,
    sn: u32,

    initial_length: usize,
}

impl<'a> Batch<'a> {
    pub fn new(data: &'a mut [u8], sn: u32) -> Self {
        let writer = data;
        Self {
            initial_length: writer.len(),

            writer,
            frame: None,
            sn,
        }
    }

    pub fn with_msg(mut self, x: &NetworkBody, r: Reliability, qos: QoS) -> ZCodecResult<Self> {
        if self.frame != Some(r) {
            <_ as ZEncode>::z_encode(
                &FrameHeader {
                    reliability: r,
                    sn: self.sn,
                    qos,
                },
                &mut self.writer,
            )?;

            self.sn += 1;
            self.frame = Some(r);
        }

        <_ as ZEncode>::z_encode(x, &mut self.writer)?;

        Ok(self)
    }

    pub fn finalize(self) -> (u32, usize) {
        (self.sn, self.initial_length - self.writer.len())
    }
}
