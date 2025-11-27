use crate::{
    Reliability, ZEncode, ZWriter,
    network::{NetworkBody, QoS},
    transport::{frame::FrameHeader, init::InitSyn, keepalive::KeepAlive, open::OpenSyn},
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

    pub fn write_init_syn(&mut self, x: &InitSyn) -> crate::ZResult<(), crate::ZCodecError> {
        <_ as ZEncode>::z_encode(x, &mut self.writer)?;
        self.frame = None;
        Ok(())
    }

    pub fn write_open_syn(&mut self, x: &OpenSyn) -> crate::ZResult<(), crate::ZCodecError> {
        <_ as ZEncode>::z_encode(x, &mut self.writer)?;
        self.frame = None;
        Ok(())
    }

    pub fn write_keepalive(&mut self) -> crate::ZResult<(), crate::ZCodecError> {
        <_ as ZEncode>::z_encode(&KeepAlive {}, &mut self.writer)?;
        self.frame = None;
        Ok(())
    }

    pub fn write_msg(
        &mut self,
        x: &NetworkBody,
        r: Reliability,
        qos: QoS,
    ) -> crate::ZResult<(), crate::ZCodecError> {
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

        Ok(())
    }

    pub fn has_written(&self) -> bool {
        self.initial_length != self.writer.len()
    }

    pub fn finalize(self) -> (u32, usize) {
        (self.sn, self.initial_length - self.writer.len())
    }
}
