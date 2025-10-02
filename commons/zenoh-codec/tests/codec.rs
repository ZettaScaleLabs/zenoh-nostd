use heapless::Vec;
use rand::{
    distributions::{Alphanumeric, DistString},
    thread_rng, Rng,
};
use uhlc::Timestamp;
use zenoh_buffer::{ZBuf, ZBufMut};
use zenoh_codec::{LCodec, RCodec, WCodec, ZCodec};
use zenoh_protocol::core::{encoding::Encoding, locator::Locator, ZenohIdProto};
use zenoh_result::{zctx, WithContext, ZResult};

mod rand_protocol;
use rand_protocol::*;

static ZCODEC: ZCodec = ZCodec;

macro_rules! run {
    ( $( ($ty:ty, $rty:ty, $rand:expr, $compared:expr) ),+ $(,)? ) => {{

        let mut data = [0u8; u8::MAX as usize];
        let mut zbuf = ZBufMut(&mut data);

        {
            let mut writer = zbuf.writer();
            $(
                let value: $ty = $rand;
                ZCODEC.write(value, &mut writer).ctx(zctx!())?;
            )+
        }

        {
            let mut reader = zbuf.reader();
            $(
                let original: $rty = $compared;
                let decoded: $rty = ZCODEC.read(&mut reader).ctx(zctx!())?;
                assert_eq!(decoded, original);
            )+
        }

        ZResult::Ok(())
    }};
}

#[test]
fn codec_string() -> ZResult<()> {
    let mut rng = rand::thread_rng();
    let str = Alphanumeric.sample_string(&mut rng, u8::MAX as usize - 2);

    run!((&str, &str, str.as_str(), str.as_str()),)
}

#[test]
fn codec_strings() -> ZResult<()> {
    let mut rng = rand::thread_rng();
    let str1 = Alphanumeric.sample_string(&mut rng, 16);
    let str2 = Alphanumeric.sample_string(&mut rng, 27);
    let str3 = Alphanumeric.sample_string(&mut rng, 6);

    run!(
        (&str, &str, str1.as_str(), str1.as_str()),
        (&str, &str, str2.as_str(), str2.as_str()),
        (&str, &str, str3.as_str(), str3.as_str()),
    )
}

#[test]
fn codec_int() -> ZResult<()> {
    let mut rng = rand::thread_rng();
    let u8r: u8 = rng.gen();
    let u16r: u16 = rng.gen();
    let u32r: u32 = rng.gen();
    let u64r: u64 = rng.gen();
    let usizer: usize = rng.gen();

    for i in 1..=ZCODEC.w_len(u64::MAX) {
        let v = { 1 << (7 * i) };
        run!((u64, u64, v, v))?;
    }

    run!(
        (u8, u8, u8::MAX, u8::MAX),
        (u8, u8, u8::MIN, u8::MIN),
        (u8, u8, u8r, u8r),
        //
        (u16, u16, u16::MAX, u16::MAX),
        (u16, u16, u16::MIN, u16::MIN),
        (u16, u16, u16r, u16r),
        //
        (u32, u32, u32::MAX, u32::MAX),
        (u32, u32, u32::MIN, u32::MIN),
        (u32, u32, u32r, u32r),
        //
        (u64, u64, u64::MAX, u64::MAX),
        (u64, u64, u64::MIN, u64::MIN),
        (u64, u64, u64r, u64r),
        //
        (usize, usize, usize::MAX, usize::MAX),
        (usize, usize, usize::MIN, usize::MIN),
        (usize, usize, usizer, usizer),
    )
}

#[test]
fn codec_zid() -> ZResult<()> {
    let zidr = ZenohIdProto::default();

    run!((&ZenohIdProto, ZenohIdProto, &zidr, zidr))
}

#[test]
fn codec_zbuf() -> ZResult<()> {
    let mut rng = rand::thread_rng();
    let len: usize = rng.gen_range(1..=u8::MAX as usize - 2);
    let data: Vec<u8, 256> = (0..len).map(|_| rng.gen()).collect();
    let zbuf = zenoh_buffer::ZBuf(&data);

    run!((&ZBuf, ZBuf, &zbuf, zbuf))?;

    Ok(())
}

#[test]
fn codec_locator() -> ZResult<()> {
    let zlocator = Locator::<64>::rand(None)?;

    run!((&Locator<64>, Locator<64>, &zlocator, zlocator),)
}

#[test]
fn codec_timestamp() -> ZResult<()> {
    let ztimestamp = Timestamp::new(
        uhlc::NTP64(thread_rng().gen()),
        uhlc::ID::try_from(ZenohIdProto::rand().to_le_bytes()).unwrap(),
    );

    run!((&Timestamp, Timestamp, &ztimestamp, ztimestamp),)
}

#[test]
fn codec_encoding() -> ZResult<()> {
    let mut data = [0u8; 32];
    let zencoding = Encoding::rand(Some(&mut data))?;

    run!((&Encoding, Encoding, &zencoding, zencoding))
}
