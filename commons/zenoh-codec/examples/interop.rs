#![no_std]

use core::str::FromStr;

use heapless::String;
use zenoh_buffer::{ZBuf, ZBufMut};
use zenoh_codec::{RCodec, WCodec, Zenoh080};
use zenoh_result::{zctx, WithContext, ZResult};

fn encode_new_codec(data: &mut [u8]) -> ZResult<()> {
    let codec = Zenoh080;
    let mut buffer = ZBufMut(data);
    let mut writer = buffer.writer();

    codec.write(42u8, &mut writer).ctx(zctx!())?;
    codec.write(4242u64, &mut writer).ctx(zctx!())?;
    let sub = [10, 20, 30, 40, 50u8];
    let zbuf = ZBuf(&sub);
    codec.write(zbuf, &mut writer).ctx(zctx!())?;
    codec.write("hello", &mut writer).ctx(zctx!())?;
    let string = String::<8>::from_str("world").unwrap();
    codec.write(&string, &mut writer).ctx(zctx!())?;

    Ok(())
}

// fn encode_official_codec(data: &mut [u8]) -> ZResult<()> {
//     use zzenoh_codec::WCodec;
//     use zzenoh_codec::Zenoh080;

//     extern crate alloc;
//     use alloc::string::String;

//     let codec = Zenoh080;
//     let mut writer = data;

//     codec.write(&mut writer, 42u8).unwrap();
//     codec.write(&mut writer, 4242u64).unwrap();
//     let sub = [10, 20, 30, 40, 50u8];
//     codec.write(&mut writer, &sub[..]).unwrap();
//     codec.write(&mut writer, "hello").unwrap();
//     let string = String::from("world");
//     codec.write(&mut writer, &string).unwrap();

//     Ok(())
// }

fn decode_new_codec(data: &[u8]) -> ZResult<()> {
    let codec = Zenoh080;
    let buffer = ZBuf(data);
    let mut reader = buffer.reader();

    let v1: u8 = codec.read(&mut reader).ctx(zctx!())?;
    let v2: u64 = codec.read(&mut reader).ctx(zctx!())?;
    let v3: ZBuf = codec.read(&mut reader).ctx(zctx!())?;
    let v4: String<6> = codec.read(&mut reader).ctx(zctx!())?;
    let v5: &str = codec.read(&mut reader).ctx(zctx!())?;

    assert_eq!(v1, 42);
    assert_eq!(v2, 4242);
    assert_eq!(v3, ZBuf(&[10, 20, 30, 40, 50u8]));
    assert_eq!(v4, String::<6>::from_str("hello").unwrap());
    assert_eq!(v5, String::<8>::from_str("world").unwrap());

    Ok(())
}

// fn decode_official_codec(data: &[u8]) -> ZResult<()> {
//     use zzenoh_codec::RCodec;
//     use zzenoh_codec::Zenoh080;

//     extern crate alloc;
//     use alloc::string::String;
//     use alloc::vec::Vec;

//     let codec = Zenoh080;
//     let mut reader = data;

//     let v1: u8 = codec.read(&mut reader).unwrap();
//     let v2: u64 = codec.read(&mut reader).unwrap();
//     let v3: Vec<u8> = codec.read(&mut reader).unwrap();
//     let v4: String = codec.read(&mut reader).unwrap();
//     let v5: String = codec.read(&mut reader).unwrap();

//     assert_eq!(v1, 42);
//     assert_eq!(v2, 4242);
//     assert_eq!(v3, Vec::from([10, 20, 30, 40, 50u8]));
//     assert_eq!(v4, String::from("hello"));
//     assert_eq!(v5, String::from("world"));

//     Ok(())
// }

fn res_main() -> ZResult<()> {
    // let mut data = [0u8; 128];
    // encode_new_codec(&mut data).ctx(zctx!())?;
    // decode_official_codec(&data).ctx(zctx!())?;

    // let mut data = [0u8; 128];
    // encode_official_codec(&mut data).ctx(zctx!())?;
    // decode_new_codec(&data).ctx(zctx!())?;

    let mut data = [0u8; 128];
    encode_new_codec(&mut data).ctx(zctx!())?;
    decode_new_codec(&data).ctx(zctx!())?;

    // let mut data = [0u8; 128];
    // encode_official_codec(&mut data).ctx(zctx!())?;
    // decode_official_codec(&data).ctx(zctx!())?;

    Ok(())
}

fn main() {
    extern crate std;

    if let Err(e) = res_main() {
        std::println!("Error: {e}");
    }
}
