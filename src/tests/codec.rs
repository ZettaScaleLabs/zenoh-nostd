extern crate std;

use crate::{
    protocol::{
        core::{encoding::*, wire_expr::WireExpr, *},
        network::{
            self,
            declare::{keyexpr::*, queryable::*, subscriber::*, *},
            interest::*,
            push::*,
            request::*,
            response::*,
            *,
        },
        transport::{
            self, TransportBody, TransportMessage, TransportSn, frame::*, init::*, keepalive::*,
            open::*,
        },
        zcodec::*,
        zenoh::{PushBody, err::*, put::*, query::*, reply::*},
    },
    zbuf::*,
};

use criterion::Criterion;
use heapless::Vec;
use rand::{Rng, thread_rng};
use std::*;
use uhlc::Timestamp;

use rand::distributions::{Alphanumeric, DistString};

#[test]
fn codec_zbuf() {
    let mut buffer = [0u8; 64];
    let mut zbuf = buffer.as_mut_slice();
    let mut writer = zbuf.writer();

    let mut rng = rand::thread_rng();
    let data: [u8; 16] = rng.r#gen();

    encode_zbuf(&mut writer, true, &data).unwrap();

    let mut reader = zbuf.reader();
    let ret = decode_zbuf(&mut reader, None).unwrap();
    assert_eq!(ret, data);
}

const NUM_ITER: usize = 100;
const MAX_PAYLOAD_SIZE: usize = 256;

use paste::paste;

#[test]
fn codec_zint() {
    macro_rules! run {
        ($ty:ident, $rand:expr) => {
            for _ in 0..NUM_ITER {
                let mut buffer = [0u8; MAX_PAYLOAD_SIZE];
                let mut zbuf = buffer.as_mut_slice();
                let mut writer = zbuf.writer();

                let value: $ty = $rand;
                paste! {
                    [<encode_ $ty>](&mut writer, value).unwrap();
                }

                let mut reader = zbuf.reader();
                let ret = paste! {
                    [<decode_ $ty>](&mut reader).unwrap()
                };
                assert_eq!(value, ret);
            }
        };
    }

    run!(u8, { u8::MIN });
    run!(u8, { u8::MAX });
    run!(u8, { thread_rng().r#gen::<u8>() });

    run!(u16, { u16::MIN });
    run!(u16, { u16::MAX });
    run!(u16, { thread_rng().r#gen::<u16>() });

    run!(u32, { u32::MIN });
    run!(u32, { u32::MAX });
    run!(u32, { thread_rng().r#gen::<u32>() });

    run!(u64, { u64::MIN });
    run!(u64, { u64::MAX });

    for i in 1..=encoded_len_u64(u64::MAX) {
        run!(u64, { 1 << (7 * i) });
    }
    run!(u64, { thread_rng().r#gen::<u64>() });

    run!(usize, { usize::MIN });
    run!(usize, { usize::MAX });
    run!(usize, { thread_rng().r#gen::<usize>() });
}

#[test]
fn codec_zint_len() {
    let mut buffer = [0u8; 16];
    let mut zbuf = buffer.as_mut_slice();
    let mut writer = zbuf.writer();
    encode_u64(&mut writer, 0).unwrap();
    assert_eq!(encoded_len_u64(0), 16 - writer.len());

    for i in 1..=encoded_len_u64(u64::MAX) {
        let mut buffer = [0u8; 16];
        let mut zbuf = buffer.as_mut_slice();
        let mut writer = zbuf.writer();
        encode_u64(&mut writer, 1 << (7 * i)).unwrap();
        assert_eq!(encoded_len_u64(1 << (7 * i)), 16 - writer.len());
    }

    let mut buffer = [0u8; 16];
    let mut zbuf = buffer.as_mut_slice();
    let mut writer = zbuf.writer();
    encode_u64(&mut writer, u64::MAX).unwrap();
    assert_eq!(encoded_len_u64(u64::MAX), 16 - writer.len());
}

#[test]
fn codec_string() {
    macro_rules! run {
        () => {
            for _ in 0..NUM_ITER {
                let mut rng = thread_rng();

                let mut buffer = [0u8; MAX_PAYLOAD_SIZE];
                let mut zbuf = buffer.as_mut_slice();
                let mut writer = zbuf.writer();

                let value: std::string::String = Alphanumeric
                    .sample_string(&mut thread_rng(), rng.gen_range(0..MAX_PAYLOAD_SIZE / 2));

                encode_str(&mut writer, true, &value).unwrap();

                let mut reader = zbuf.reader();
                let ret = decode_str(&mut reader, None).unwrap();
                assert_eq!(value, ret);
            }
        };
    }

    run!()
}

macro_rules! run {
    (args, $ty:ty, ($($rand_args:expr),*), ($($enc_args:expr),*), ($($dec_args:expr),*)) => {
        for _ in 0..NUM_ITER {
            let mut buffer = [0u8; MAX_PAYLOAD_SIZE];
            let mut zbuf = buffer.as_mut_slice();
            let mut writer = zbuf.writer();

            let value = <$ty>::rand($($rand_args),*);
            value.encode($($enc_args),*, &mut writer).unwrap();

            let mut reader = zbuf.reader();
            let ret = <$ty>::decode($($dec_args),*, &mut reader).unwrap();
            assert_eq!(value, ret);
        }
    };

    (no_enc, $ty:ty, ($($rand_args:expr),*), ($($dec_args:expr),*)) => {
        for _ in 0..NUM_ITER {
            let mut buffer = [0u8; MAX_PAYLOAD_SIZE * 16];
            let mut zbuf = buffer.as_mut_slice();
            let mut writer = zbuf.writer();

            let value = <$ty>::rand($($rand_args),*);
            value.encode(&mut writer).unwrap();

            let mut reader = zbuf.reader();
            let ret = <$ty>::decode($($dec_args),*, &mut reader).unwrap();
            assert_eq!(value, ret);
        }
    };


    (header, $ty:ty, ($($rand_args:expr),*)) => {
        for _ in 0..NUM_ITER {
            let mut buffer = [0u8; MAX_PAYLOAD_SIZE * 8];
            let mut zbuf = buffer.as_mut_slice();
            let mut writer = zbuf.writer();

            let value = <$ty>::rand($($rand_args),*);
            value.encode(&mut writer).unwrap();

            let mut reader = zbuf.reader();
            let header = decode_u8(&mut reader).unwrap();
            let ret = <$ty>::decode(header, &mut reader).unwrap();
            assert_eq!(value, ret);
        }
    };

    (no_args, $ty:ty, ($($rand_args:expr),*)) => {
        for _ in 0..NUM_ITER {
            let mut buffer = [0u8; MAX_PAYLOAD_SIZE];
            let mut zbuf = buffer.as_mut_slice();
            let mut writer = zbuf.writer();

            let value = <$ty>::rand($($rand_args),*);
            value.encode(&mut writer).unwrap();

            let mut reader = zbuf.reader();
            let ret = <$ty>::decode(&mut reader).unwrap();
            assert_eq!(value, ret);
        }
    };

    (no_rand, $ty:ty, $rand:expr) => {
        for _ in 0..NUM_ITER {
            let mut buffer = [0u8; MAX_PAYLOAD_SIZE];
            let mut zbuf = buffer.as_mut_slice();
            let mut writer = zbuf.writer();

            let value: $ty = $rand;
            value.encode(&mut writer).unwrap();

            let mut reader = zbuf.reader();
            let ret = <$ty>::decode(&mut reader).unwrap();
            assert_eq!(value, ret);
        }
    }
}

#[test]
fn codec_zid() {
    run!(args, ZenohIdProto, (), (true), (None))
}

#[test]
fn codec_timestamp() {
    for _ in 0..NUM_ITER {
        let mut buffer = [0u8; MAX_PAYLOAD_SIZE];
        let mut zbuf = buffer.as_mut_slice();
        let mut writer = zbuf.writer();

        let value: Timestamp = {
            let time = uhlc::NTP64(thread_rng().r#gen());
            let id = uhlc::ID::try_from(ZenohIdProto::rand().as_le_bytes()).unwrap();
            Timestamp::new(time, id)
        };
        encode_timestamp(&mut writer, &value).unwrap();

        let mut reader = zbuf.reader();
        let ret = decode_timestamp(&mut reader).unwrap();
        assert_eq!(value, ret);
    }
}

#[test]
fn codec_encoding() {
    let mut buffer = [0u8; MAX_PAYLOAD_SIZE * NUM_ITER];
    let mut zbuf = buffer.as_mut_slice();
    let mut r_writer = zbuf.writer();

    run!(no_args, Encoding, (&mut r_writer));
}

#[test]
fn codec_extension() {
    macro_rules! run_extension {
        ($ext:ty, ($($rand_args:expr),*)) => {
            for _ in 0..NUM_ITER {
                let mut buffer = [0u8; MAX_PAYLOAD_SIZE];
                let mut zbuf = buffer.as_mut_slice();
                let mut writer = zbuf.writer();

                let more: bool = thread_rng().r#gen();
                let x = <$ext>::rand($($rand_args),*);

                x.encode(more, &mut writer).unwrap();

                let mut reader = zbuf.reader();
                let header = decode_u8(&mut reader).unwrap();
                let y: ($ext, bool) = <$ext>::decode(header, &mut reader).unwrap();

                assert_eq!(x, y.0);
                assert_eq!(more, y.1);
            }
        };
    }

    let mut buffer = [0u8; MAX_PAYLOAD_SIZE * NUM_ITER];
    let mut zbuf = buffer.as_mut_slice();
    let mut r_writer = zbuf.writer();

    run_extension!(crate::zextz64!(0x01, true), ());
    run_extension!(crate::zextz64!(0x01, false), ());
    run_extension!(crate::zextzbuf!('_, 0x02, true), (&mut r_writer));
    run_extension!(crate::zextzbuf!('_, 0x02, false), (&mut r_writer));
}

#[test]
fn codec_init_syn() {
    let mut buffer = [0u8; MAX_PAYLOAD_SIZE * NUM_ITER];
    let mut zbuf = buffer.as_mut_slice();
    let mut r_writer = zbuf.writer();

    run!(header, InitSyn, (&mut r_writer));
}

#[test]
fn codec_init_ack() {
    let mut buffer = [0u8; MAX_PAYLOAD_SIZE * NUM_ITER];
    let mut zbuf = buffer.as_mut_slice();
    let mut r_writer = zbuf.writer();

    run!(header, InitAck, (&mut r_writer));
}

#[test]
fn codec_open_syn() {
    let mut buffer = [0u8; MAX_PAYLOAD_SIZE * NUM_ITER];
    let mut zbuf = buffer.as_mut_slice();
    let mut r_writer = zbuf.writer();

    run!(header, OpenSyn, (&mut r_writer));
}

#[test]
fn codec_open_ack() {
    let mut buffer = [0u8; MAX_PAYLOAD_SIZE * NUM_ITER];
    let mut zbuf = buffer.as_mut_slice();
    let mut r_writer = zbuf.writer();

    run!(header, OpenAck, (&mut r_writer));
}

#[test]
fn codec_keep_alive() {
    run!(header, KeepAlive, ());
}

#[test]
fn codec_frame_header() {
    run!(header, FrameHeader, ());
}

#[test]
fn codec_declare() {
    let mut buffer = [0u8; MAX_PAYLOAD_SIZE * NUM_ITER];
    let mut zbuf = buffer.as_mut_slice();
    let mut r_writer = zbuf.writer();

    run!(header, Declare, (&mut r_writer));
}

#[test]
fn codec_declare_body() {
    let mut buffer = [0u8; MAX_PAYLOAD_SIZE * NUM_ITER];
    let mut zbuf = buffer.as_mut_slice();
    let mut r_writer = zbuf.writer();

    run!(no_args, DeclareBody, (&mut r_writer));
}

#[test]
fn codec_interest() {
    let mut buffer = [0u8; MAX_PAYLOAD_SIZE * NUM_ITER];
    let mut zbuf = buffer.as_mut_slice();
    let mut r_writer = zbuf.writer();

    run!(header, Interest, (&mut r_writer));
}

#[test]
fn codec_declare_keyexpr() {
    let mut buffer = [0u8; MAX_PAYLOAD_SIZE * NUM_ITER];
    let mut zbuf = buffer.as_mut_slice();
    let mut r_writer = zbuf.writer();

    run!(header, DeclareKeyExpr, (&mut r_writer));
}

#[test]
fn codec_undeclare_keyexpr() {
    run!(header, UndeclareKeyExpr, ());
}

#[test]
fn codec_declare_subscriber() {
    let mut buffer = [0u8; MAX_PAYLOAD_SIZE * NUM_ITER];
    let mut zbuf = buffer.as_mut_slice();
    let mut r_writer = zbuf.writer();

    run!(header, DeclareSubscriber, (&mut r_writer));
}

#[test]
fn codec_undeclare_subscriber() {
    let mut buffer = [0u8; MAX_PAYLOAD_SIZE * NUM_ITER];
    let mut zbuf = buffer.as_mut_slice();
    let mut r_writer = zbuf.writer();

    run!(header, UndeclareSubscriber, (&mut r_writer));
}

#[test]
fn codec_declare_queryable() {
    let mut buffer = [0u8; MAX_PAYLOAD_SIZE * NUM_ITER];
    let mut zbuf = buffer.as_mut_slice();
    let mut r_writer = zbuf.writer();

    run!(header, DeclareQueryable, (&mut r_writer));
}

#[test]
fn codec_undeclare_queryable() {
    let mut buffer = [0u8; MAX_PAYLOAD_SIZE * NUM_ITER];
    let mut zbuf = buffer.as_mut_slice();
    let mut r_writer = zbuf.writer();

    run!(header, UndeclareQueryable, (&mut r_writer));
}

#[test]
fn codec_push() {
    let mut buffer = [0u8; u16::MAX as usize];
    let mut zbuf = buffer.as_mut_slice();
    let mut r_writer = zbuf.writer();

    run!(header, Push, (&mut r_writer));
}

#[test]
fn codec_request() {
    let mut buffer = [0u8; MAX_PAYLOAD_SIZE * NUM_ITER * 8];
    let mut zbuf = buffer.as_mut_slice();
    let mut r_writer = zbuf.writer();

    run!(header, Request, (&mut r_writer));
}

#[test]
fn codec_response() {
    let mut buffer = [0u8; MAX_PAYLOAD_SIZE * NUM_ITER * 8];
    let mut zbuf = buffer.as_mut_slice();
    let mut r_writer = zbuf.writer();

    run!(header, Response, (&mut r_writer));
}

#[test]
fn codec_response_final() {
    run!(header, ResponseFinal, ());
}

#[test]
fn codec_network() {
    let mut buffer = [0u8; MAX_PAYLOAD_SIZE * NUM_ITER * 8];
    let mut zbuf = buffer.as_mut_slice();
    let mut r_writer = zbuf.writer();

    run!(
        no_enc,
        NetworkMessage,
        (&mut r_writer),
        (Reliability::DEFAULT)
    );
}

// Zenoh
#[test]
fn codec_put() {
    let mut buffer = [0u8; MAX_PAYLOAD_SIZE * NUM_ITER * 8];
    let mut zbuf = buffer.as_mut_slice();
    let mut r_writer = zbuf.writer();

    run!(header, Put, (&mut r_writer));
}

#[test]
fn codec_query() {
    let mut buffer = [0u8; MAX_PAYLOAD_SIZE * NUM_ITER * 8];
    let mut zbuf = buffer.as_mut_slice();
    let mut r_writer = zbuf.writer();

    run!(header, Query, (&mut r_writer));
}

#[test]
fn codec_reply() {
    let mut buffer = [0u8; MAX_PAYLOAD_SIZE * NUM_ITER * 8];
    let mut zbuf = buffer.as_mut_slice();
    let mut r_writer = zbuf.writer();

    run!(header, Reply, (&mut r_writer));
}

#[test]
fn codec_err() {
    let mut buffer = [0u8; MAX_PAYLOAD_SIZE * NUM_ITER * 8];
    let mut zbuf = buffer.as_mut_slice();
    let mut r_writer = zbuf.writer();

    run!(header, Err, (&mut r_writer));
}
#[test]
fn codec_frame() {
    let mut buffer = [0u8; MAX_PAYLOAD_SIZE * NUM_ITER * 8];
    let mut zbuf = buffer.as_mut_slice();
    let mut r_writer = zbuf.writer();

    let mut r_vec = Vec::<NetworkMessage<'_>, 16>::new();

    for _ in 0..NUM_ITER {
        let mut buffer = [0u8; MAX_PAYLOAD_SIZE * 16];
        let mut zbuf = buffer.as_mut_slice();
        let mut writer = zbuf.writer();

        let value = Frame::rand(&mut r_writer, &mut r_vec);
        let value2 = value.clone();
        let value = TransportMessage {
            body: TransportBody::Frame(value),
        };

        value.encode(&mut writer).unwrap();

        let mut i = 0;
        let mut reader = zbuf.reader();
        TransportMessage::decode_batch(
            &mut reader,
            None::<fn(InitSyn)>,
            None::<fn(InitAck)>,
            None::<fn(OpenSyn)>,
            None::<fn(OpenAck)>,
            None::<fn()>,
            Some(|_: &FrameHeader, msg: NetworkMessage<'_>| {
                assert_eq!(msg, value2.payload[i]);

                i += 1;
            }),
        )
        .unwrap();
    }
}

pub(super) fn criterion(c: &mut Criterion) {
    let mut buff = [0u8; 16];
    let mut zbuf: ZBufMut = buff.as_mut_slice();
    c.bench_function("Encode u64", |b| {
        b.iter(|| {
            let mut writer = zbuf.writer();
            encode_u64(&mut writer, u64::MAX).unwrap();
            let mut reader = zbuf.reader();
            let _: u64 = decode_u64(&mut reader).unwrap();
        })
    });

    let mut buff = [0u8; 64];
    let mut zbuf: ZBufMut = buff.as_mut_slice();
    c.bench_function("Encode b'Hello, world!'", |b| {
        b.iter(|| {
            let mut writer = zbuf.writer();
            encode_str(&mut writer, true, "Hello, world!").unwrap();
            let mut reader = zbuf.reader();
            let _: &str = decode_str(&mut reader, None).unwrap();
        })
    });

    let mut buff = [0u8; u16::MAX as usize];

    let frame = FrameHeader {
        reliability: Reliability::DEFAULT,
        sn: TransportSn::MIN,
        ext_qos: transport::frame::ext::QoSType::DEFAULT,
    };

    let msg = NetworkMessage {
        reliability: Reliability::BestEffort,
        body: NetworkBody::Push(Push {
            wire_expr: WireExpr::from("demo/example"),
            ext_qos: network::push::ext::QoSType::DEFAULT,
            ext_nodeid: network::push::ext::NodeIdType::DEFAULT,
            ext_tstamp: None,
            payload: PushBody::Put(Put {
                timestamp: None,
                encoding: Encoding::empty(),
                ext_sinfo: None,
                ext_attachment: None,
                payload: &[0u8; 8],
            }),
        }),
    };

    let mut zbuf: ZBufMut = buff.as_mut_slice();
    c.bench_function("Encode Batch NetworkMsg", |b| {
        b.iter(|| {
            let mut writer = zbuf.writer();
            frame.encode(&mut writer).unwrap();
            while msg.encode(&mut writer).is_ok() {}
        })
    });

    let mut buff = [0u8; u16::MAX as usize];
    let mut zbuf: ZBufMut = buff.as_mut_slice();
    let mut writer = zbuf.writer();
    frame.encode(&mut writer).unwrap();
    while msg.encode(&mut writer).is_ok() {}
    c.bench_function("Decode Batch NetworkMsg", |b| {
        b.iter(|| {
            let mut reader = zbuf.reader();
            TransportMessage::decode_batch(
                &mut reader,
                None::<fn(InitSyn)>,
                None::<fn(InitAck)>,
                None::<fn(OpenSyn)>,
                None::<fn(OpenAck)>,
                None::<fn()>,
                None::<fn(&FrameHeader, NetworkMessage)>,
            )
            .unwrap();
        })
    });
}
