use zenoh_codec::{RCodec, WCodec, Zenoh080};
// use zenoh_protocol::{
//     core::{resolution::Resolution, whatami::WhatAmI, ZenohIdProto},
//     transport::{
//         init::{ext, InitSyn},
//         BatchSize,
//     },
//     VERSION,
// };

fn main() {
    let mut buffer = [0u8; 128];

    let codec = Zenoh080;

    let value = 1u8;
    let len = codec.write(&value, &mut buffer).unwrap();

    let value = 666u64;
    let _ = codec.write(&value, &mut buffer[len..]).unwrap();

    let (value_u8, len): (u8, usize) = codec.read(&buffer).unwrap();
    println!("u8: {value_u8}, len: {len}");

    let (value_u64, len): (u64, usize) = codec.read(&buffer[len..]).unwrap();
    println!("u64: {value_u64}, len: {len}");
}
