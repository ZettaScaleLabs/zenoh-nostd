mod core;

mod network;
mod transport;
mod zenoh;

mod ke;

macro_rules! roundtrip {
    ($ty:ty) => {{
        let mut rand_data = [0u8; MAX_PAYLOAD_SIZE * NUM_ITER];
        let mut rand_writer = rand_data.as_mut_slice();

        let mut data = [0u8; MAX_PAYLOAD_SIZE];
        for _ in 0..NUM_ITER {
            let mut writer = data.as_mut_slice();
            let value = <$ty>::rand(&mut rand_writer);
            let len = <_ as $crate::ZLen>::z_len(&value);
            <_ as $crate::ZEncode>::z_encode(&value, &mut writer).unwrap();
            let mut reader = data.as_slice();
            let ret = <$ty as $crate::ZDecode>::z_decode(
                &mut <_ as $crate::ZReaderExt>::sub(&mut reader, len).unwrap(),
            )
            .unwrap();
            assert_eq!(ret, value);
        }
    }};

    (ext, $ty:ty) => {{
        let mut rand_data = [0u8; MAX_PAYLOAD_SIZE * NUM_ITER];
        let mut rand_writer = rand_data.as_mut_slice();

        let mut data = [0u8; MAX_PAYLOAD_SIZE];
        for _ in 0..NUM_ITER {
            let mut writer = data.as_mut_slice();
            let value = <$ty>::rand(&mut rand_writer);
            $crate::zext_encode::<_, 0x1, true>(&value, &mut writer, false).unwrap();
            let mut reader = data.as_slice();
            let ret = $crate::zext_decode::<$ty>(&mut reader).unwrap();
            assert_eq!(ret, value);
        }
    }};
}
pub(crate) use roundtrip;

macro_rules! roundtrips {
    (ext, $namespace:ident, $($ty:ty),* $(,)?) => {
        $(
            paste::paste! {
                #[test]
                fn [<$namespace _proto_ext_ $ty:lower>]() {
                    $crate::tests::protocol::roundtrip!(ext, $ty);
                }
            }
        )*
    };

    ($namespace:ident, $($ty:ty),* $(,)?) => {
        $(
            paste::paste! {
                #[test]
                fn [<$namespace _proto_ $ty:lower>]() {
                    $crate::tests::protocol::roundtrip!($ty);
                }
            }
        )*
    };
}
pub(crate) use roundtrips;
