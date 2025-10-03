use zenoh_buffer::ZBufReader;
use zenoh_protocol::{
    common::imsg,
    network::NetworkMessage,
    transport::{frame::FrameHeader, id, TransportBody},
};
use zenoh_result::{zctx, WithContext, ZResult};

use crate::{RCodec, ZCodec};

fn handle_frame<'a>(
    codec: &ZCodec,
    reader: &mut ZBufReader<'a>,
    header: u8,
    mut on_network_msg: impl FnMut(FrameHeader, NetworkMessage<'a>) -> ZResult<()>,
) -> ZResult<()> {
    let header: FrameHeader = codec.read_knowing_header(reader, header)?;

    while reader.can_read() {
        let mark = reader.mark();
        let msg: ZResult<NetworkMessage<'a>> = codec
            .read_with_reliability(reader, header.reliability)
            .ctx(zctx!());

        match msg {
            Ok(msg) => {
                on_network_msg(header, msg)?;
            }
            Err(_) => {
                reader.rewind(mark).unwrap();
                break;
            }
        }
    }

    Ok(())
}

impl ZCodec {
    pub fn read_batch<'a>(
        &self,
        reader: &mut ZBufReader<'a>,
        mut on_network_msg: impl FnMut(FrameHeader, NetworkMessage<'a>) -> ZResult<()>,
        mut on_other: impl FnMut(TransportBody<'a>) -> ZResult<()>,
    ) -> ZResult<()> {
        while reader.can_read() {
            let Ok(header): ZResult<u8> = self.read(reader) else {
                break;
            };

            match imsg::mid(header) {
                id::FRAME => handle_frame(self, reader, header, &mut on_network_msg)?,
                id::KEEP_ALIVE => {
                    let body = TransportBody::KeepAlive(self.read_knowing_header(reader, header)?);
                    on_other(body)?;
                    continue;
                }
                id::INIT => {
                    if !imsg::has_flag(header, zenoh_protocol::transport::init::flag::A) {
                        let body =
                            TransportBody::InitSyn(self.read_knowing_header(reader, header)?);
                        on_other(body)?;
                        continue;
                    } else {
                        let body =
                            TransportBody::InitAck(self.read_knowing_header(reader, header)?);
                        on_other(body)?;
                        continue;
                    }
                }
                id::OPEN => {
                    if !imsg::has_flag(header, zenoh_protocol::transport::open::flag::A) {
                        let body =
                            TransportBody::OpenSyn(self.read_knowing_header(reader, header)?);
                        on_other(body)?;
                        continue;
                    } else {
                        let body =
                            TransportBody::OpenAck(self.read_knowing_header(reader, header)?);
                        on_other(body)?;
                        continue;
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }
}
