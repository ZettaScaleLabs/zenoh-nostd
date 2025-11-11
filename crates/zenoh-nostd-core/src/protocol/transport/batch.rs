use crate::{
    Reliability, ZCodecResult, ZEncode, ZWriter,
    network::{NetworkBody, QoS},
    transport::{
        close::Close,
        frame::FrameHeader,
        init::{InitAck, InitSyn},
        keepalive::KeepAlive,
        open::{OpenAck, OpenSyn},
    },
};

pub struct Batch<'a> {
    writer: ZWriter<'a>,
    frame: Option<Reliability>,
    sn: u32,

    initial_length: usize,
}

impl<'a> Batch<'a> {
    pub fn new(data: &'a mut [u8], sn: u32) -> Self {
        let writer = data.as_mut();
        Self {
            initial_length: writer.len(),

            writer,
            frame: None,
            sn,
        }
    }

    pub fn with_init_syn(mut self, x: &InitSyn) -> ZCodecResult<Self> {
        <_ as ZEncode>::z_encode(x, &mut self.writer)?;
        Ok(self)
    }

    pub fn with_init_ack(mut self, x: &InitAck) -> ZCodecResult<Self> {
        <_ as ZEncode>::z_encode(x, &mut self.writer)?;
        Ok(self)
    }

    pub fn with_open_syn(mut self, x: &OpenSyn) -> ZCodecResult<Self> {
        <_ as ZEncode>::z_encode(x, &mut self.writer)?;
        Ok(self)
    }

    pub fn with_open_ack(mut self, x: &OpenAck) -> ZCodecResult<Self> {
        <_ as ZEncode>::z_encode(x, &mut self.writer)?;
        Ok(self)
    }

    pub fn with_close(mut self, x: &Close) -> ZCodecResult<Self> {
        <_ as ZEncode>::z_encode(x, &mut self.writer)?;
        Ok(self)
    }

    pub fn with_keepalive(mut self, x: &KeepAlive) -> ZCodecResult<Self> {
        <_ as ZEncode>::z_encode(x, &mut self.writer)?;
        Ok(self)
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

    pub fn len(&self) -> usize {
        self.initial_length - self.writer.len()
    }
}
