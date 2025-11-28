use crate::{exts::*, msgs::*, *};

pub struct BatchWriter<'a> {
    writer: ZWriter<'a>,
    frame: Option<Reliability>,
    sn: u32,

    initial_length: usize,
}

impl<'a> BatchWriter<'a> {
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

    pub fn write_init_ack(&mut self, x: &InitAck) -> crate::ZResult<(), crate::ZCodecError> {
        <_ as ZEncode>::z_encode(x, &mut self.writer)?;
        self.frame = None;
        Ok(())
    }

    pub fn write_open_syn(&mut self, x: &OpenSyn) -> crate::ZResult<(), crate::ZCodecError> {
        <_ as ZEncode>::z_encode(x, &mut self.writer)?;
        self.frame = None;
        Ok(())
    }

    pub fn write_open_ack(&mut self, x: &OpenAck) -> crate::ZResult<(), crate::ZCodecError> {
        <_ as ZEncode>::z_encode(x, &mut self.writer)?;
        self.frame = None;
        Ok(())
    }

    pub fn write_keepalive(&mut self) -> crate::ZResult<(), crate::ZCodecError> {
        <_ as ZEncode>::z_encode(&KeepAlive {}, &mut self.writer)?;
        self.frame = None;
        Ok(())
    }

    pub fn write_close(&mut self, x: &Close) -> crate::ZResult<(), crate::ZCodecError> {
        <_ as ZEncode>::z_encode(x, &mut self.writer)?;
        self.frame = None;
        Ok(())
    }

    pub fn write_msg(
        &mut self,
        x: &FrameBody,
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

#[derive(Debug, PartialEq)]
pub enum TransportBody<'a, 'b> {
    InitSyn(InitSyn<'a>),
    InitAck(InitAck<'a>),
    OpenSyn(OpenSyn<'a>),
    OpenAck(OpenAck<'a>),
    Close(Close),
    KeepAlive(KeepAlive),
    Frame(Frame<'a, 'b>),
}

pub struct BatchReader<'a> {
    reader: ZReader<'a>,
}

impl<'a> BatchReader<'a> {
    pub fn new(reader: ZReader<'a>) -> BatchReader<'a> {
        BatchReader { reader }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<ZResult<TransportBody<'a, '_>, ZCodecError>> {
        if !self.reader.can_read() {
            return None;
        }

        let mark = self.reader.mark();
        let header = self
            .reader
            .read_u8()
            .expect("reader should not be empty at this stage");

        macro_rules! decode {
            ($ty:ty) => {
                match <$ty as $crate::ZBodyDecode>::z_body_decode(&mut self.reader, header) {
                    Ok(msg) => msg,
                    Err(err) => {
                        return Some(Err(err));
                    }
                }
            };
        }

        let ack = header & 0b0010_0000 != 0;
        let body = match header & 0b0001_1111 {
            InitAck::ID if ack => TransportBody::InitAck(decode!(InitAck)),
            InitSyn::ID => TransportBody::InitSyn(decode!(InitSyn)),
            OpenAck::ID if ack => TransportBody::OpenAck(decode!(OpenAck)),
            OpenSyn::ID => TransportBody::OpenSyn(decode!(OpenSyn)),
            Close::ID => TransportBody::Close(decode!(Close)),
            KeepAlive::ID => TransportBody::KeepAlive(decode!(KeepAlive)),
            FrameHeader::ID => {
                let frame = decode!(FrameHeader);
                TransportBody::Frame(Frame {
                    header: frame,
                    msgs: &mut self.reader,
                })
            }
            _ => {
                self.reader.rewind(mark);
                return None;
            }
        };

        Some(Ok(body))
    }
}
