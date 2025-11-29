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
    InterestFinal {
        frame: FrameHeader,
        body: InterestFinal,
    },
    Declare {
        frame: FrameHeader,
        body: Declare<'a>,
    },
}

pub struct ZBatchReader<'a, T> {
    reader: T,
    _lt: ::core::marker::PhantomData<&'a ()>,
    frame: Option<FrameHeader>,
}

impl<'a, T> ZBatchReader<'a, T>
where
    T: crate::ZRead<'a>,
{
    pub fn new(reader: T) -> Self {
        Self {
            reader,
            _lt: ::core::marker::PhantomData,
            frame: None,
        }
    }
}

impl<'a, T> Iterator for ZBatchReader<'a, T>
where
    T: crate::ZRead<'a>,
{
    type Item = ZMessage<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.reader.can_read() {
            return None;
        }

        let header = self
            .reader
            .read_u8()
            .expect("reader should not be empty at this stage");

        macro_rules! decode {
            ($ty:ty) => {
                match <$ty as $crate::ZBodyDecode>::z_body_decode(&mut self.reader, header) {
                    Ok(msg) => msg,
                    Err(_) => {
                        return None;
                    }
                }
            };
        }

        let ack = header & 0b0010_0000 != 0;
        let net = self.frame.is_some();
        let ifinal = header & 0b0110_0000 == 0;

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
            InterestFinal::ID if net && ifinal => ZMessage::InterestFinal {
                frame: self
                    .frame
                    .expect("Should be a frame. Something went wrong."),
                body: decode!(InterestFinal),
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
                return None;
            }
        };

        Some(body)
    }
}

pub struct ZBatchWriter<'a, T> {
    writer: T,
    _lt: ::core::marker::PhantomData<&'a ()>,
    frame: Option<FrameHeader>,
    sn: u32,

    init: usize,
}

impl<'a, T> ZBatchWriter<'a, T>
where
    T: crate::ZWrite,
{
    pub fn new(writer: T, sn: u32) -> Self {
        let init = writer.remaining();
        Self {
            writer,
            _lt: ::core::marker::PhantomData,
            frame: None,
            sn,
            init,
        }
    }

    pub fn has_written(&self) -> bool {
        self.init != self.writer.remaining()
    }

    pub fn finalize(self) -> (u32, usize) {
        (self.sn, self.init - self.writer.remaining())
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

impl<'a, T: Unframed, W> ZBatchUnframed<T> for ZBatchWriter<'a, W>
where
    W: crate::ZWrite,
{
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

impl<'a, T: Framed, W> ZBatchFramed<T> for ZBatchWriter<'a, W>
where
    W: crate::ZWrite,
{
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
