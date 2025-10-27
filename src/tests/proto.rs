use zenoh_proto::ZExt;

use crate::{
    protocol::{
        codec::{encoded_len_u32, encoded_len_u64},
        core::encoding::Encoding,
        ext::{ZExt, ZExtKind},
    },
    zbuf::{ZBuf, ZBufExt, ZBufMutExt},
};

#[test]
fn test_unit() {
    #[derive(ZExt)]
    struct Unit1;
    #[derive(ZExt)]
    struct Unit2 {}
    #[derive(ZExt)]
    struct Unit3();

    assert!(<Unit1 as ZExt>::KIND == ZExtKind::Unit);
    assert!(<Unit2 as ZExt>::KIND == ZExtKind::Unit);
    assert!(<Unit3 as ZExt>::KIND == ZExtKind::Unit);
}

#[test]
#[allow(dead_code)]
fn test_u64() {
    #[derive(ZExt)]
    struct U641 {
        #[u8]
        field1: u8,
    }

    #[derive(ZExt)]
    struct U642 {
        #[u16]
        field1: u16,
    }

    #[derive(ZExt)]
    struct U643 {
        #[u32]
        field1: u32,
    }

    #[derive(ZExt)]
    struct U644 {
        #[u64]
        field1: u64,
    }

    #[derive(ZExt)]
    struct U645 {
        #[usize]
        field1: usize,
    }

    assert!(<U641 as ZExt>::KIND == ZExtKind::U64);
    assert!(<U642 as ZExt>::KIND == ZExtKind::U64);
    assert!(<U643 as ZExt>::KIND == ZExtKind::U64);
    assert!(<U644 as ZExt>::KIND == ZExtKind::U64);
    assert!(<U645 as ZExt>::KIND == ZExtKind::U64);

    #[derive(ZExt)]
    struct U646(#[u8] u8);
    #[derive(ZExt)]
    struct U647(#[u16] u16);
    #[derive(ZExt)]
    struct U648(#[u32] u32);
    #[derive(ZExt)]
    struct U649(#[u64] u64);
    #[derive(ZExt)]
    struct U6410(#[usize] usize);

    assert!(<U646 as ZExt>::KIND == ZExtKind::U64);
    assert!(<U647 as ZExt>::KIND == ZExtKind::U64);
    assert!(<U648 as ZExt>::KIND == ZExtKind::U64);
    assert!(<U649 as ZExt>::KIND == ZExtKind::U64);
    assert!(<U6410 as ZExt>::KIND == ZExtKind::U64);

    #[derive(ZExt)]
    struct U6411(#[u32] u32, #[u16] u16);
    #[derive(ZExt)]
    struct U6412 {
        #[u16]
        field1: u16,
        #[u32]
        field2: u32,
    }
    #[derive(ZExt)]
    struct U6413 {
        #[u8]
        field1: u8,
        #[u8]
        field2: u8,
        #[u16]
        field3: u16,
    }

    assert!(<U6411 as ZExt>::KIND == ZExtKind::U64);
    assert!(<U6412 as ZExt>::KIND == ZExtKind::U64);
    assert!(<U6413 as ZExt>::KIND == ZExtKind::U64);

    let mut data = [0u8; 10];
    let mut zbuf = data.as_mut_slice();
    let mut writer = zbuf.writer();

    let value = U6413 {
        field1: 255,
        field2: 20,
        field3: u16::MAX,
    };

    <U6413 as ZExt>::ENCODE(&mut writer, &value).unwrap();
    let mut reader = data.as_slice().reader();
    let decoded = <U6413 as ZExt>::DECODE(&mut reader, 0).unwrap();

    assert_eq!(value.field1, decoded.field1);
    assert_eq!(value.field2, decoded.field2);
    assert_eq!(value.field3, decoded.field3);
}

#[test]
#[allow(dead_code)]
fn test_zbuf() {
    #[derive(ZExt)]
    struct ZBuf1<'a>(#[zbuf(deduced)] ZBuf<'a>);

    #[derive(ZExt)]
    struct ZBuf2<'a> {
        #[zbuf(plain)]
        field1: ZBuf<'a>,
    }

    assert!(<ZBuf1 as ZExt>::KIND == ZExtKind::ZBuf);
    assert!(<ZBuf2 as ZExt>::KIND == ZExtKind::ZBuf);

    #[derive(ZExt)]
    struct ZBuf3 {
        #[u32]
        field1: u32,
        #[u64]
        field2: u64,
    }

    assert!(<ZBuf3 as ZExt>::KIND == ZExtKind::ZBuf);

    #[derive(ZExt)]
    struct ZBuf4<'a> {
        #[u16]
        field1: u16,
        #[zbuf(flag = 4)]
        field2: ZBuf<'a>,
        #[zbuf(flag = 4)]
        field3: ZBuf<'a>,
        #[zbuf(plain)]
        field4: ZBuf<'a>,
        #[zbuf(deduced)]
        field5: ZBuf<'a>,
    }

    #[derive(ZExt)]
    struct ZBuf5<'a>(#[zbuf(plain)] ZBuf<'a>, #[zbuf(deduced)] ZBuf<'a>);

    assert!(<ZBuf4 as ZExt>::KIND == ZExtKind::ZBuf);
    assert!(<ZBuf5 as ZExt>::KIND == ZExtKind::ZBuf);

    #[derive(ZExt)]
    struct ZBuf6<'a> {
        #[zbuf(plain)]
        field1: ZBuf<'a>,
        #[composite(crate::protocol::core::encoding, encoding)]
        field2: Encoding<'a>,
        #[u8]
        field3: u8,
    }

    assert!(<ZBuf6 as ZExt>::KIND == ZExtKind::ZBuf);

    let array1 = [1u8, 2, 3, 4, 5];
    let array2 = [10u8, 20, 30, 40, 50, 60, 70, 80];
    let array3 = [100u8, 101, 102];
    let array4 = [200u8, 201, 202, 203, 204, 205];

    let zbuf1 = array1.as_slice();
    let zbuf2 = array2.as_slice();
    let zbuf3 = array3.as_slice();
    let zbuf4 = array4.as_slice();

    let zb = ZBuf4 {
        field1: 42,
        field2: zbuf1,
        field3: zbuf2,
        field4: zbuf3,
        field5: zbuf4,
    };
    let len = <ZBuf4 as ZExt>::LEN(&zb);

    let mut data = [0u8; 100];
    let mut zbuf = data.as_mut_slice();
    let mut writer = zbuf.writer();

    <ZBuf4 as ZExt>::ENCODE(&mut writer, &zb).unwrap();

    let mut reader = data.as_slice().reader();
    let decoded = <ZBuf4 as ZExt>::DECODE(&mut reader, len).unwrap();

    assert_eq!(zb.field1, decoded.field1);
    assert_eq!(zb.field1, decoded.field1);
    assert_eq!(zb.field2, decoded.field2);
    assert_eq!(zb.field3, decoded.field3);
    assert_eq!(zb.field4, decoded.field4);
    assert_eq!(zb.field5, decoded.field5);
}
