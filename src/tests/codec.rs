extern crate std;
use std::string::{String, ToString};

use crate::{
    protocol::{
        ZCodecError,
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
    result::ZResult,
    zbuf::*,
};

use criterion::Criterion;
use heapless::Vec;
use paste::paste;
use rand::{Rng, thread_rng};
use std::*;
use uhlc::Timestamp;

use rand::distributions::{Alphanumeric, DistString};

#[allow(non_snake_case)]
pub(crate) fn encode_String(
    writer: &mut ZBufWriter<'_>,
    s: String,
    len: bool,
) -> ZResult<(), ZCodecError> {
    encode_str(writer, s.as_str(), len)
}

#[allow(non_snake_case)]
pub(crate) fn decode_String<'a>(
    reader: &mut ZBufReader<'a>,
    len: Option<usize>,
) -> ZResult<String, ZCodecError> {
    decode_str(reader, len).map(|s| s.to_string())
}

#[allow(non_snake_case)]
pub(crate) fn encode_Timestamp(
    writer: &mut ZBufWriter<'_>,
    x: Timestamp,
) -> ZResult<(), ZCodecError> {
    encode_timestamp(writer, &x)
}

#[allow(non_snake_case)]
pub(crate) fn decode_Timestamp<'a>(reader: &mut ZBufReader<'a>) -> ZResult<Timestamp, ZCodecError> {
    decode_timestamp(reader)
}

const NUM_ITER: usize = 100;
const MAX_PAYLOAD_SIZE: usize = 512;

macro_rules! run {
    (full,
        $ty:ty,
        RAND: $rand_fct:expr,
        ENCODE: $enc_fct:expr,
        DECODE: $dec_fct:expr,
        ASSERT: $assert_fct:expr
    ) => {
        let mut r_buffer = [0u8; MAX_PAYLOAD_SIZE * NUM_ITER];
        let mut r_zbuf = r_buffer.as_mut_slice();
        let mut r_writer = r_zbuf.writer();

        let mut buffer = [0u8; MAX_PAYLOAD_SIZE];
        let mut zbuf = buffer.as_mut_slice();

        for _ in 0..NUM_ITER {
            #[allow(unused_variables, unused_mut)]
            let mut writer = zbuf.writer();
            let value: $ty = $rand_fct(&mut r_writer);
            $enc_fct(writer, &value).unwrap();

            #[allow(unused_variables, unused_mut)]
            let mut reader = zbuf.reader();
            let ret = $dec_fct(reader).unwrap();
            $assert_fct(value, ret);
        }
    };

    (args,
        $ty:ty,
        ENCODE: ( $( $enc_args:expr ),* ),
        DECODE: ( $( $dec_args:expr ),* )
    ) => {
        run!(
            full,
            $ty,
            RAND: |mut w| <$ty>::rand(&mut w),
            ENCODE: |mut w, v: &$ty| v.encode(&mut w, $( $enc_args ),*),
            DECODE: |mut r| <$ty>::decode(&mut r, $( $dec_args ),*),
            ASSERT: |a: $ty, b: $ty| assert_eq!(a, b)
        );
    };

    (default,
        $ty:ty
    ) => {
        run!(
            args,
            $ty,
            ENCODE: (),
            DECODE: ()
        );
    };

    (header,
        $ty:ty
    ) => {
        run!(
            full,
            $ty,
            RAND: |mut w| <$ty>::rand(&mut w),
            ENCODE: |mut w, v: &$ty| v.encode(&mut w),
            DECODE: |mut r| {
                let header = decode_u8(&mut r).unwrap();
                <$ty>::decode(&mut r, header)
            },
            ASSERT: |a: $ty, b: $ty| assert_eq!(a, b)
        );
    };

    (raw_args,
        $ty:ty,
        RAND: $rand:expr,
        ENCODE: ( $( $enc_args:expr ),* ),
        DECODE: ( $( $dec_args:expr ),* )
    ) => {
        run!(
            full,
            $ty,
            RAND: |_| $rand,
            ENCODE: |mut w, v: &$ty| paste! { [<encode_ $ty>](&mut w, v.clone(), $( $enc_args ),*) },
            DECODE: |mut r| paste! { [<decode_ $ty>](&mut r, $( $dec_args ),*) },
            ASSERT: |a: $ty, b: $ty| assert_eq!(a, b)
        );
    };

    (raw,
        $ty:ty,
        RAND: $rand:expr
    ) => {
        run!(
            raw_args,
            $ty,
            RAND: $rand,
            ENCODE: (),
            DECODE: ()
        );
    };

    (ext,
        $ty:ty)
    => {
        let mut t: bool = false;
        run!(
            full,
            $ty,
            RAND: |mut w| <$ty>::rand(&mut w),
            ENCODE: |mut w, v: &$ty| {
                t = thread_rng().r#gen();
                v.encode(&mut w, t)
            },
            DECODE: |mut r| {
                let header = decode_u8(&mut r).unwrap();
                <$ty>::decode(&mut r, header)
            },
            ASSERT: |a: $ty, b: ($ty, bool)| {
                assert_eq!(a, b.0);
                assert_eq!(t, b.1);
            }
        );
    }
}

macro_rules! test_header {
    ($( $ty:ty ),+ ) => {
        $(
            paste! {
                #[test]
                fn [<codec_ $ty:lower>]() {
                    run!(header, $ty);
                }
            }
        )+
    };
}

test_header!(
    InitSyn,
    InitAck,
    OpenSyn,
    OpenAck,
    KeepAlive,
    FrameHeader,
    Declare,
    Interest,
    DeclareKeyExpr,
    UndeclareKeyExpr,
    DeclareSubscriber,
    UndeclareSubscriber,
    DeclareQueryable,
    UndeclareQueryable,
    Push,
    Request,
    Response,
    ResponseFinal,
    Put,
    Query,
    Reply,
    Err
);

#[test]
fn codec_encoding() {
    run!(default, Encoding);
}

#[test]
fn codec_declare_body() {
    run!(default, DeclareBody);
}

#[test]
fn codec_network() {
    run!(args, NetworkMessage, ENCODE: (), DECODE: (Reliability::DEFAULT));
}

#[test]
fn codec_string() {
    run!(raw_args, String,
        RAND: Alphanumeric
            .sample_string(&mut thread_rng(), thread_rng().gen_range(0..MAX_PAYLOAD_SIZE / 2)),
        ENCODE: (true),
        DECODE: (None)
    );
}

#[test]
fn codec_zid() {
    run!(args, ZenohIdProto, ENCODE: (true), DECODE: (None));
}

#[test]
fn codec_timestamp() {
    run!(raw, Timestamp, RAND: {
        let time = uhlc::NTP64(thread_rng().r#gen());
        let id = uhlc::ID::try_from(ZenohIdProto::default().as_le_bytes()).unwrap();
        Timestamp::new(time, id)
    });
}

#[test]
fn codec_extension() {
    run!(ext, crate::zextz64!(0x01, true));
    run!(ext, crate::zextz64!(0x01, false));
    run!(ext, crate::zextzbuf!('_, 0x02, true));
    run!(ext, crate::zextzbuf!('_, 0x02, false));
}

#[test]
fn codec_zbuf() {
    let mut buffer = [0u8; 64];
    let mut zbuf = buffer.as_mut_slice();
    let mut writer = zbuf.writer();

    let mut rng = rand::thread_rng();
    let data: [u8; 16] = rng.r#gen();

    encode_zbuf(&mut writer, &data, true).unwrap();

    let mut reader = zbuf.reader();
    let ret = decode_zbuf(&mut reader, None).unwrap();
    assert_eq!(ret, data);
}

#[test]
fn codec_zint() {
    macro_rules! run_int {
        ( $( $ty:ident ),+ ) => {
            $(
                run!(raw, $ty, RAND: $ty::MIN);
                run!(raw, $ty, RAND: $ty::MAX);
                run!(raw, $ty, RAND: thread_rng().r#gen::<$ty>());
            )+
        };
    }

    run_int!(u8, u16, u32, u64, usize);

    for i in 1..=encoded_len_u64(u64::MAX) {
        run!(raw, u64, RAND: 1 << (7 * i));
    }
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
            encode_str(&mut writer, "Hello, world!", true).unwrap();
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
