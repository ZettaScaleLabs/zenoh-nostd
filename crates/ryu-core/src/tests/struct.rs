use core::fmt::Debug;

use crate::marker;
use crate::{ByteReaderExt, ZStruct};

#[derive(ZStruct, PartialEq, Debug)]
struct ZBasic {
    pub id: u8,
    pub value: u32,
    #[size(none)]
    pub array: [u8; 4],
}

#[derive(ZStruct, PartialEq, Debug)]
struct ZWithLifetime<'a> {
    pub sn: u16,
    #[size(plain)]
    pub data: &'a [u8],
}

#[derive(ZStruct, PartialEq, Debug)]
struct ZOptionPlain<'a> {
    #[option(plain)]
    pub maybe_u32: Option<u32>,

    #[option(plain, size(plain))]
    pub maybe_str: Option<&'a str>,
}

#[derive(ZStruct, PartialEq, Debug)]
struct ZOptionFlag<'a> {
    _flag: marker::Flag,

    #[option(flag)]
    pub maybe_byte: Option<u8>,

    #[option(flag, size(flag = 5))]
    pub maybe_str: Option<&'a str>,
}

#[derive(ZStruct, PartialEq, Debug)]
struct ZOptionFlagDeduced<'a> {
    _flag: marker::Flag,

    #[option(flag, size(flag = 7))]
    pub maybe_slice: Option<&'a [u8]>,

    #[size(deduced)]
    pub trailing_data: &'a str,
}

mod nested {
    use super::*;
    #[derive(ZStruct, PartialEq, Debug)]
    pub struct Inner {
        pub a: u32,
        pub b: u8,
    }
}

#[derive(ZStruct, PartialEq, Debug)]
struct ZNested {
    pub field1: nested::Inner,
    pub field2: nested::Inner,
}

#[derive(ZStruct, PartialEq, Debug)]
struct ZNestedOption<'a> {
    #[option(plain)]
    pub maybe_inner: Option<nested::Inner>,

    #[size(plain)]
    pub name: &'a str,
}

#[derive(ZStruct, PartialEq, Debug)]
struct ZFlagComplex<'a> {
    _flag: marker::Flag,

    #[option(flag)]
    pub maybe_u8: Option<u8>,

    #[option(flag, size(flag = 2))]
    pub maybe_slice: Option<&'a [u8]>,

    #[option(flag, size(eflag = 3))]
    pub maybe_str: Option<&'a str>,

    #[size(plain)]
    pub payload: u64,
}

#[derive(ZStruct, PartialEq, Debug)]
struct ZArrays<'a> {
    pub fixed_array: [u8; 8],

    #[size(plain)]
    pub slice_plain: &'a [u8],

    #[size(deduced)]
    pub no_size_str: &'a str,
}

#[derive(ZStruct, PartialEq, Debug)]
struct ZFlags<'a> {
    _flag: marker::Flag,

    #[option(flag)]
    pub small_opt: Option<u8>,

    #[option(flag, size(flag = 5))]
    pub mid_opt: Option<&'a str>,

    #[option(flag, size(deduced))]
    pub big_opt: Option<&'a [u8]>,
}

mod deep {
    use super::*;
    #[derive(ZStruct, PartialEq, Debug)]
    pub struct Inner<'a> {
        pub seq: u32,
        #[size(plain)]
        pub data: &'a [u8],
    }
}

#[derive(ZStruct, PartialEq, Debug)]
struct ZComplex<'a> {
    pub id: u32,
    pub qos: u8,
    _flag: marker::Flag,

    #[option(flag)]
    pub opt_int: Option<u16>,

    #[option(flag, size(flag = 6))]
    pub opt_str: Option<&'a str>,

    #[option(plain)]
    pub opt_inner: Option<deep::Inner<'a>>,

    #[size(plain)]
    pub inner: deep::Inner<'a>,

    #[size(deduced)]
    pub trailing: &'a str,
}

#[derive(ZStruct, PartialEq, Debug)]
struct ZHeader<'a> {
    _header: marker::Header,

    #[hstore(value = 0b1010_0000)]
    _id: marker::Phantom,

    #[hstore(mask = 0b0001_1100, shift = 2)]
    pub vu8: u8,

    #[hstore(mask = 0b0000_0011, shift = 0)]
    pub vu8_2: u8,

    #[option(header = 0b1000_0000, size(plain))]
    pub keyexpr: Option<&'a str>,

    #[size(plain)]
    pub field1: deep::Inner<'a>,

    #[option(header = 0b0100_0000, size(deduced))]
    pub field2: Option<ZComplex<'a>>,
}

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
}

#[test]
fn test_zbasic() {
    let s = ZBasic {
        id: 42,
        value: 123456,
        array: [1, 2, 3, 4],
    };
    roundtrip!(ZBasic, s);
}

#[test]
fn test_zwitlifetime() {
    let buf = [10, 20, 30];
    let s = ZWithLifetime { sn: 11, data: &buf };
    roundtrip!(ZWithLifetime, s);
}

#[test]
fn test_zoption_plain() {
    let s = ZOptionPlain {
        maybe_u32: Some(99),
        maybe_str: Some("hello"),
    };
    roundtrip!(ZOptionPlain, s);

    let s2 = ZOptionPlain {
        maybe_u32: None,
        maybe_str: None,
    };
    roundtrip!(ZOptionPlain, s2);
}

#[test]
fn test_zoption_flag() {
    let s = ZOptionFlag {
        _flag: marker::Flag,
        maybe_byte: Some(7),
        maybe_str: Some("flagged"),
    };
    roundtrip!(ZOptionFlag, s);
}

#[test]
fn test_zoption_flag_deduced() {
    let buf = [1, 2, 3];
    let s = ZOptionFlagDeduced {
        _flag: marker::Flag,
        maybe_slice: Some(&buf),
        trailing_data: "xyz",
    };
    roundtrip!(ZOptionFlagDeduced, s);
}

#[test]
fn test_znested() {
    let s = ZNested {
        field1: nested::Inner { a: 1, b: 2 },
        field2: nested::Inner { a: 3, b: 4 },
    };
    roundtrip!(ZNested, s);
}

#[test]
fn test_znested_option() {
    let s = ZNestedOption {
        maybe_inner: Some(nested::Inner { a: 42, b: 7 }),
        name: "nested",
    };
    roundtrip!(ZNestedOption, s);
}

#[test]
fn test_zflag_complex() {
    let buf = [5, 6, 7];
    let s = ZFlagComplex {
        _flag: marker::Flag,
        maybe_u8: Some(1),
        maybe_slice: Some(&buf),
        maybe_str: Some("hi"),
        payload: 123456789,
    };
    roundtrip!(ZFlagComplex, s);
}

#[test]
fn test_zarrays() {
    let fixed = [9; 8];
    let s = ZArrays {
        fixed_array: fixed,
        slice_plain: &[1, 2, 3],
        no_size_str: "str",
    };
    roundtrip!(ZArrays, s);
}

#[test]
fn test_zmultiple_flags() {
    let buf = [42; 4];
    let s = ZFlags {
        _flag: marker::Flag,
        small_opt: Some(5),
        mid_opt: Some("flagged"),
        big_opt: Some(&buf),
    };
    roundtrip!(ZFlags, s);
}

#[test]
fn test_zcomplex() {
    let buf = [1, 2, 3, 4];
    let opt_inner = deep::Inner {
        seq: 42,
        data: &buf,
    };

    let inner = deep::Inner {
        seq: 99,
        data: &buf,
    };
    let s = ZComplex {
        id: 1,
        qos: 2,
        _flag: marker::Flag,
        opt_int: Some(123),
        opt_str: Some("hello"),
        opt_inner: Some(opt_inner),
        inner,
        trailing: "world",
    };
    roundtrip!(ZComplex, s);
}

#[test]
fn test_zheader() {
    let buf = [1, 2, 3, 4];
    let opt_inner = deep::Inner {
        seq: 42,
        data: &buf,
    };

    let inner = deep::Inner {
        seq: 99,
        data: &buf,
    };
    let s = ZComplex {
        id: 1,
        qos: 2,
        _flag: marker::Flag,
        opt_int: Some(123),
        opt_str: Some("hello"),
        opt_inner: Some(opt_inner),
        inner,
        trailing: "world",
    };
    let header = ZHeader {
        _header: marker::Header,
        _id: marker::Phantom,
        vu8: 0b0000_0101,
        vu8_2: 0b0000_0011,
        keyexpr: Some("key.expr"),
        field1: deep::Inner { seq: 7, data: &buf },
        field2: Some(s),
    };

    roundtrip!(ZHeader, header);
}
