use crate::{exts::*, msgs::*, *};

#[derive(ZEnum, Debug, PartialEq)]
pub enum FrameBody<'a> {
    Push(Push<'a>),
    Request(Request<'a>),
    Response(Response<'a>),
    ResponseFinal(ResponseFinal),
    Interest(Interest<'a>),
    Declare(Declare<'a>),
}

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|_|R|ID:5=0x05")]
pub struct FrameHeader {
    #[zenoh(header = R)]
    pub reliability: Reliability,
    pub sn: u32,

    #[zenoh(ext = 0x1, default = QoS::DEFAULT)]
    pub qos: QoS,
}

#[derive(Debug, PartialEq)]
pub struct Frame<'a, 'b> {
    pub header: FrameHeader,
    pub msgs: &'b mut ZReader<'a>,
}

impl<'a, 'b> ::core::iter::Iterator for Frame<'a, 'b> {
    type Item = ZResult<FrameBody<'a>, ZCodecError>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.msgs.can_read() {
            return None;
        }

        let mark = self.msgs.mark();
        let header = self
            .msgs
            .read_u8()
            .expect("reader should not be empty at this stage");

        macro_rules! decode {
            ($ty:ty) => {{
                match <$ty as $crate::ZBodyDecode>::z_body_decode(self.msgs, header) {
                    Ok(msg) => msg,
                    Err(err) => {
                        return Some(Err(err));
                    }
                }
            }};
        }

        let body = match header & 0b0001_1111 {
            Push::ID => FrameBody::Push(decode!(Push)),
            Request::ID => FrameBody::Request(decode!(Request)),
            Response::ID => FrameBody::Response(decode!(Response)),
            ResponseFinal::ID => FrameBody::ResponseFinal(decode!(ResponseFinal)),
            Interest::ID => FrameBody::Interest(decode!(Interest)),
            Declare::ID => FrameBody::Declare(decode!(Declare)),
            _ => {
                self.msgs.rewind(mark);
                return None;
            }
        };

        Some(Ok(body))
    }
}

impl Drop for Frame<'_, '_> {
    fn drop(&mut self) {
        for _ in self {}
    }
}
