use crate::{
    Reliability, ZCodecError, ZEncode, ZReader, ZReaderExt, ZResult, ZWriter, network::*,
    transport::*,
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

#[derive(Debug, PartialEq)]
pub struct NetworkBatch<'a, 'b> {
    pub reader: &'b mut ZReader<'a>,
}

impl<'a, 'b> NetworkBatch<'a, 'b> {
    pub fn new(reader: &'b mut ZReader<'a>) -> Self {
        Self { reader }
    }
}

impl<'a, 'b> core::iter::Iterator for NetworkBatch<'a, 'b> {
    type Item = ZResult<NetworkBody<'a>, ZCodecError>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.reader.can_read() {
            return None;
        }

        let mark = self.reader.mark();
        let header = self
            .reader
            .read_u8()
            .expect("reader should not be empty at this stage");

        macro_rules! decode {
            ($ty:ty) => {{
                match <$ty as $crate::ZBodyDecode>::z_body_decode(self.reader, header) {
                    Ok(msg) => msg,
                    Err(err) => {
                        return Some(Err(err));
                    }
                }
            }};
        }

        let body = match header & 0b0001_1111 {
            Push::ID => NetworkBody::Push(decode!(Push)),
            Request::ID => NetworkBody::Request(decode!(Request)),
            Response::ID => NetworkBody::Response(decode!(Response)),
            ResponseFinal::ID => NetworkBody::ResponseFinal(decode!(ResponseFinal)),
            Interest::ID => NetworkBody::Interest(decode!(Interest)),
            Declare::ID => NetworkBody::Declare(decode!(Declare)),
            _ => {
                self.reader.rewind(mark);
                return None;
            }
        };

        Some(Ok(body))
    }
}

pub struct TransportBatch<'a> {
    reader: ZReader<'a>,
}

impl<'a> TransportBatch<'a> {
    pub fn new(reader: ZReader<'a>) -> TransportBatch<'a> {
        TransportBatch { reader }
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
                let iter = NetworkBatch::new(&mut self.reader);
                TransportBody::Frame(Frame {
                    header: frame,
                    msgs: iter,
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
