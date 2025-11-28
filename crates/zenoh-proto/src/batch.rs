use crate::{exts::*, fields::*, msgs::*, *};

#[derive(Debug, PartialEq)]
pub enum ZMessage<'a> {
    Close(Close),
    InitSyn(InitSyn<'a>),
    InitAck(InitAck<'a>),
    KeepAlive(KeepAlive),
    OpenSyn(OpenSyn<'a>),
    OpenAck(OpenAck<'a>),

    Push {
        frame: FrameHeader,
        body: Push<'a>,
    },
    Request {
        frame: FrameHeader,
        body: Request<'a>,
    },
    Response {
        frame: FrameHeader,
        body: Response<'a>,
    },
    ResponseFinal {
        frame: FrameHeader,
        body: ResponseFinal,
    },
    Interest {
        frame: FrameHeader,
        body: Interest<'a>,
    },
    Declare {
        frame: FrameHeader,
        body: Declare<'a>,
    },
}

pub struct ZBatchReader<'a> {
    reader: ZReader<'a>,
    frame: Option<FrameHeader>,
}

impl<'a> ZBatchReader<'a> {
    pub fn new(reader: ZReader<'a>) -> Self {
        Self {
            reader,
            frame: None,
        }
    }
}

impl<'a> Iterator for ZBatchReader<'a> {
    type Item = ZMessage<'a>;

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
            ($ty:ty) => {
                match <$ty as $crate::ZBodyDecode>::z_body_decode(&mut self.reader, header) {
                    Ok(msg) => msg,
                    Err(err) => {
                        crate::error!(
                            "Failed to decode message of type {}: {:?}",
                            stringify!($ty),
                            err
                        );

                        self.reader.rewind(mark);
                        return None;
                    }
                }
            };
        }

        let ack = header & 0b0010_0000 != 0;
        let net = self.frame.is_some();

        let body = match header & 0b0001_1111 {
            InitAck::ID if ack => ZMessage::InitAck(decode!(InitAck)),
            InitSyn::ID => ZMessage::InitSyn(decode!(InitSyn)),
            OpenAck::ID if ack => ZMessage::OpenAck(decode!(OpenAck)),
            OpenSyn::ID => ZMessage::OpenSyn(decode!(OpenSyn)),
            Close::ID => ZMessage::Close(decode!(Close)),
            KeepAlive::ID => ZMessage::KeepAlive(decode!(KeepAlive)),

            FrameHeader::ID => {
                let frame = decode!(FrameHeader);
                self.frame = Some(frame);
                return self.next();
            }
            Push::ID if net => ZMessage::Push {
                frame: self
                    .frame
                    .expect("Should be a frame. Something went wrong."),
                body: decode!(Push),
            },
            Request::ID if net => ZMessage::Request {
                frame: self
                    .frame
                    .expect("Should be a frame. Something went wrong."),
                body: decode!(Request),
            },
            Response::ID if net => ZMessage::Response {
                frame: self
                    .frame
                    .expect("Should be a frame. Something went wrong."),
                body: decode!(Response),
            },
            ResponseFinal::ID if net => ZMessage::ResponseFinal {
                frame: self
                    .frame
                    .expect("Should be a frame. Something went wrong."),
                body: decode!(ResponseFinal),
            },
            Interest::ID if net => ZMessage::Interest {
                frame: self
                    .frame
                    .expect("Should be a frame. Something went wrong."),
                body: decode!(Interest),
            },
            Declare::ID if net => ZMessage::Declare {
                frame: self
                    .frame
                    .expect("Should be a frame. Something went wrong."),
                body: decode!(Declare),
            },

            _ => {
                crate::error!("Unexpected message type: header={:#04x}", header);

                self.reader.rewind(mark);
                return None;
            }
        };

        Some(body)
    }
}

pub struct ZBatchWriter<'a> {
    writer: ZWriter<'a>,
    frame: Option<FrameHeader>,
    sn: u32,

    init: usize,
}

impl<'a> ZBatchWriter<'a> {
    pub fn new(writer: ZWriter<'a>, sn: u32) -> Self {
        let init = writer.len();
        Self {
            writer,
            frame: None,
            sn,
            init,
        }
    }

    pub fn has_written(&self) -> bool {
        self.init != self.writer.len()
    }

    pub fn finalize(self) -> (u32, usize) {
        (self.sn, self.init - self.writer.len())
    }
}

pub trait Unframed: ZEncode {}

impl Unframed for InitSyn<'_> {}
impl Unframed for InitAck<'_> {}
impl Unframed for OpenSyn<'_> {}
impl Unframed for OpenAck<'_> {}
impl Unframed for KeepAlive {}
impl Unframed for Close {}

pub trait ZBatchUnframed<T: Unframed> {
    fn unframe(&mut self, x: &T) -> crate::ZResult<(), crate::ZCodecError>;
}

impl<'a, T: Unframed> ZBatchUnframed<T> for ZBatchWriter<'a> {
    fn unframe(&mut self, x: &T) -> crate::ZResult<(), crate::ZCodecError> {
        <_ as ZEncode>::z_encode(x, &mut self.writer)?;
        self.frame = None;
        Ok(())
    }
}

pub trait Framed: ZEncode {}

impl Framed for Push<'_> {}
impl Framed for Request<'_> {}
impl Framed for Response<'_> {}
impl Framed for ResponseFinal {}
impl Framed for Interest<'_> {}
impl Framed for Declare<'_> {}

pub trait ZBatchFramed<T: Framed> {
    fn frame(&mut self, x: &T, r: Reliability, qos: QoS) -> crate::ZResult<(), crate::ZCodecError>;
}

impl<'a, T: Framed> ZBatchFramed<T> for ZBatchWriter<'a> {
    fn frame(&mut self, x: &T, r: Reliability, qos: QoS) -> crate::ZResult<(), crate::ZCodecError> {
        if self.frame.as_ref().map(|f| f.reliability) != Some(r) {
            <_ as ZEncode>::z_encode(
                &FrameHeader {
                    reliability: r,
                    sn: self.sn,
                    qos,
                },
                &mut self.writer,
            )?;

            self.frame = Some(FrameHeader {
                reliability: r,
                sn: self.sn,
                qos,
            });

            self.sn += 1;
        }

        <_ as ZEncode>::z_encode(x, &mut self.writer)?;

        Ok(())
    }
}
