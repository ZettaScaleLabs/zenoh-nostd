use crate::{ByteReaderExt, ZExt, ZExtKind, ZStruct, marker, zextfield};

#[derive(ZExt, PartialEq, Debug)]
pub struct ZExtEmpty {}

#[derive(ZExt, PartialEq, Debug)]
pub struct ZExtCounter {
    pub counter: u64,
}

#[derive(ZExt, PartialEq, Debug)]
pub struct ZExtData<'a> {
    #[size(plain)]
    pub bytes: &'a [u8],
}

#[derive(ZExt, PartialEq, Debug)]
pub struct ZExtInfo<'a> {
    pub id: u16,
    #[size(deduced)]
    pub name: &'a str,
}

#[derive(ZExt, PartialEq, Debug)]
pub struct ZExtFlagged<'a> {
    _flag: marker::Flag,

    #[option(flag)]
    pub maybe_u8: Option<u8>,

    #[option(flag, size(deduced))]
    pub maybe_str: Option<&'a str>,
}

#[derive(ZStruct, PartialEq, Debug)]
pub struct ZMsgSimple<'a> {
    #[size(plain)]
    pub title: &'a str,

    #[option(plain)]
    pub _begin: marker::ExtBlockBegin,
    pub ext_data: Option<ZExtData<'a>>,
    pub ext_info: Option<ZExtInfo<'a>>,
    pub _end: marker::ExtBlockEnd,

    #[size(deduced)]
    pub payload: &'a [u8],
}

#[derive(ZStruct, PartialEq, Debug)]
pub struct ZMsgHeader<'a> {
    _header: marker::Header,

    #[size(plain)]
    pub topic: &'a str,

    #[option(header = 0b1000_0000)]
    pub _begin: marker::ExtBlockBegin,
    pub ext_flagged: Option<ZExtFlagged<'a>>,
    pub ext_empty: Option<ZExtEmpty>,
    pub _end: marker::ExtBlockEnd,

    pub value: u32,
}

#[derive(ZStruct, PartialEq, Debug)]
pub struct ZMsgCounters {
    _flag: marker::Flag,

    #[option(flag)]
    pub _begin: marker::ExtBlockBegin,
    pub ext1: Option<ZExtCounter>,
    pub _end: marker::ExtBlockEnd,

    pub checksum: u16,
}

#[derive(ZStruct, PartialEq, Debug)]
pub struct ZMsgComplex<'a> {
    _header: marker::Header,

    #[size(plain)]
    pub name: &'a str,

    #[option(header = 0b1000_0000)]
    pub _begin: marker::ExtBlockBegin,
    pub ext_info: Option<ZExtInfo<'a>>,
    pub ext_data: Option<ZExtData<'a>>,
    pub ext_empty: Option<ZExtEmpty>,
    pub _end: marker::ExtBlockEnd,

    #[size(deduced)]
    pub trailing: &'a str,
}

zextfield!(impl<'a> ZExtData<'a>, ZMsgSimple<'a>, 0x01, true);
zextfield!(impl<'a> ZExtInfo<'a>, ZMsgSimple<'a>, 0x02, true);

zextfield!(impl<'a> ZExtFlagged<'a>, ZMsgHeader<'a>, 0x01, true);
zextfield!(impl<'a> ZExtEmpty, ZMsgHeader<'a>, 0x02, true);

zextfield!(ZExtCounter, ZMsgCounters, 0x01, true);

zextfield!(impl<'a> ZExtInfo<'a>, ZMsgComplex<'a>, 0x01, true);
zextfield!(impl<'a> ZExtData<'a>, ZMsgComplex<'a>, 0x02, true);
zextfield!(impl<'a> ZExtEmpty, ZMsgComplex<'a>, 0x03, true);

macro_rules! roundtrip {
    ($ty:ty, $value:expr) => {{
        let mut data = [0u8; 256];
        let mut writer = &mut data.as_mut_slice();

        let len = <$ty as ZStruct>::z_len(&$value);
        <$ty as ZStruct>::z_encode(&$value, &mut writer).unwrap();

        let mut reader = data.as_slice();
        let decoded = <$ty as ZStruct>::z_decode(&mut reader.sub(len).unwrap()).unwrap();

        assert_eq!(decoded, $value);
    }};

    (ext, $ty:ty, $value:expr) => {{
        let mut data = [0u8; 256];
        let mut writer = &mut data.as_mut_slice();

        <$ty as ZExt>::z_encode(&$value, &mut writer).unwrap();

        let mut reader = data.as_slice();
        let decoded = <$ty as ZExt>::z_decode(&mut reader).unwrap();

        assert_eq!(decoded, $value);
    }};
}

#[test]
fn test_zext_kinds() {
    assert_eq!(ZExtEmpty::KIND, ZExtKind::Unit);
    assert_eq!(ZExtCounter::KIND, ZExtKind::U64);
    assert_eq!(ZExtData::KIND, ZExtKind::ZStruct);
    assert_eq!(ZExtInfo::KIND, ZExtKind::ZStruct);
    assert_eq!(ZExtFlagged::KIND, ZExtKind::ZStruct);
}

#[test]
fn test_zext_roundtrips() {
    let buf = [10, 20, 30];
    roundtrip!(ext, ZExtData, ZExtData { bytes: &buf });

    roundtrip!(
        ext,
        ZExtInfo,
        ZExtInfo {
            id: 42,
            name: "device"
        }
    );

    roundtrip!(
        ext,
        ZExtFlagged,
        ZExtFlagged {
            _flag: marker::Flag,
            maybe_u8: Some(5),
            maybe_str: Some("flagged"),
        }
    );
}

#[test]
fn test_zmsg_simple() {
    let buf = [1, 2, 3, 4];
    let msg = ZMsgSimple {
        title: "simple",
        _begin: marker::ExtBlockBegin,
        ext_data: Some(ZExtData { bytes: &buf }),
        ext_info: Some(ZExtInfo {
            id: 99,
            name: "info",
        }),
        _end: marker::ExtBlockEnd,
        payload: &buf,
    };
    roundtrip!(ZMsgSimple, msg);
}

#[test]
fn test_zmsg_header() {
    let msg = ZMsgHeader {
        _header: marker::Header,
        topic: "topic/1",
        _begin: marker::ExtBlockBegin,
        ext_flagged: Some(ZExtFlagged {
            _flag: marker::Flag,
            maybe_u8: Some(7),
            maybe_str: Some("extra"),
        }),
        ext_empty: Some(ZExtEmpty {}),
        _end: marker::ExtBlockEnd,
        value: 12345,
    };
    roundtrip!(ZMsgHeader, msg);
}

#[test]
fn test_zmsg_counters() {
    let msg = ZMsgCounters {
        _flag: marker::Flag,
        _begin: marker::ExtBlockBegin,
        ext1: Some(ZExtCounter { counter: 10 }),
        _end: marker::ExtBlockEnd,
        checksum: 55,
    };
    roundtrip!(ZMsgCounters, msg);
}

#[test]
fn test_zmsg_complex() {
    let data = [9, 9, 9];
    let msg = ZMsgComplex {
        _header: marker::Header,
        name: "complex",
        _begin: marker::ExtBlockBegin,
        ext_info: Some(ZExtInfo {
            id: 11,
            name: "ext",
        }),
        ext_data: Some(ZExtData { bytes: &data }),
        ext_empty: Some(ZExtEmpty {}),
        _end: marker::ExtBlockEnd,
        trailing: "end",
    };
    roundtrip!(ZMsgComplex, msg);
}

#[test]
fn test_zmsg_partial_exts() {
    let msg = ZMsgComplex {
        _header: marker::Header,
        name: "partial",
        _begin: marker::ExtBlockBegin,
        ext_info: None,
        ext_data: Some(ZExtData { bytes: &[1, 2, 3] }),
        ext_empty: None,
        _end: marker::ExtBlockEnd,
        trailing: "zzz",
    };
    roundtrip!(ZMsgComplex, msg);
    let msg = ZMsgComplex {
        _header: marker::Header,
        name: "partial",
        _begin: marker::ExtBlockBegin,
        ext_info: None,
        ext_data: None,
        ext_empty: None,
        _end: marker::ExtBlockEnd,
        trailing: "zzz",
    };
    roundtrip!(ZMsgComplex, msg);
}
