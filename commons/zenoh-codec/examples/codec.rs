#![no_std]

use heapless::Vec;
use zenoh_buffer::{ZBuf, ZBufMut};
use zenoh_codec::{WCodec, ZCodec};
use zenoh_protocol::{
    core::{encoding::Encoding, wire_expr::WireExpr},
    network::{push::Push, NetworkBody, NetworkMessage},
    transport::{
        frame::{Frame, FrameHeader},
        TransportBody, TransportMessage,
    },
    zenoh::{put::Put, PushBody},
};
use zenoh_result::{zctx, WithContext, ZResult};

fn network_msg(payload: ZBuf<'_>) -> NetworkMessage<'_> {
    NetworkMessage {
        reliability: zenoh_protocol::core::Reliability::BestEffort,
        body: NetworkBody::Push(Push {
            wire_expr: WireExpr::from("value"),
            ext_qos: zenoh_protocol::network::push::ext::QoSType::DEFAULT,
            ext_nodeid: zenoh_protocol::network::push::ext::NodeIdType::DEFAULT,
            ext_tstamp: None,
            payload: PushBody::Put(Put {
                timestamp: None,
                encoding: Encoding::empty(),
                ext_sinfo: None,
                ext_attachment: None,
                payload: payload,
            }),
        }),
    }
}

fn frame<'a>(msgs: &'a [NetworkMessage<'a>]) -> Frame<'a> {
    Frame {
        reliability: zenoh_protocol::core::Reliability::BestEffort,
        sn: 32,
        ext_qos: zenoh_protocol::transport::frame::ext::QoSType::DEFAULT,
        payload: msgs,
    }
}

fn transport_msg<'a>(frame: Frame<'a>) -> TransportMessage<'a> {
    TransportMessage {
        body: TransportBody::Frame(frame),
    }
}

fn res_main() -> ZResult<()> {
    extern crate std;

    let mut data = [0u8; 1500];
    {
        let payload1 = ZBuf(b"Hello, World");
        let payload2 = ZBuf(b"Another message");
        let msgs1 = [
            network_msg(payload1),
            network_msg(payload2.clone()),
            network_msg(payload2),
        ];

        let frame1 = frame(&msgs1);

        let payload3 = ZBuf(b"Third message");
        let msgs2 = [network_msg(payload3)];

        let frame2 = frame(&msgs2);

        let transportmsg1 = transport_msg(frame1);
        let transportmsg2 = transport_msg(frame2);

        let mut zbuf = ZBufMut(&mut data);
        let mut writer = zbuf.writer();

        ZCodec.write(&transportmsg1, &mut writer)?;
        let len1 = writer.pos();
        ZCodec.write(&transportmsg2, &mut writer)?;
        let len2 = writer.pos();

        std::println!(
            "Encoded data: \n{:?}\n{:?}",
            &data[..len1],
            &data[len1..len2]
        );
    }

    let zbuf = ZBuf(&data);
    let mut reader = zbuf.reader();

    let mut payloads = Vec::<ZBuf<'_>, 256>::new();

    ZCodec
        .read_batch(
            &mut reader,
            |_: FrameHeader, msg: NetworkMessage| {
                match msg.body {
                    NetworkBody::Push(push) => match push.payload {
                        PushBody::Put(put) => {
                            payloads.push(put.payload).unwrap();
                        }
                    },
                    _ => {}
                }

                Ok(())
            },
            |_: TransportBody| Ok(()),
        )
        .ctx(zctx!())?;

    for payload in payloads {
        std::println!("Decoded payload: {:?}", payload.as_str());
    }

    Ok(())
}

fn main() {
    extern crate std;

    std::process::exit(match res_main() {
        Ok(_) => 0,
        Err(e) => {
            std::eprintln!("Error: {}", e);
            1
        }
    });
}
