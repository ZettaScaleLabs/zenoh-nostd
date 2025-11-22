use crate::{ZCodecResult, ZReader, ZWriter};

mod array;
mod bytes;
mod str;
mod uint;

pub trait ZBodyLen {
    fn z_body_len(&self) -> usize;
}

pub trait ZBodyEncode {
    fn z_body_encode(&self, w: &mut ZWriter) -> ZCodecResult<()>;
}

pub trait ZBodyDecode<'a>: Sized {
    type Ctx;

    fn z_body_decode(r: &mut ZReader<'a>, ctx: Self::Ctx) -> ZCodecResult<Self>;
}

pub trait ZHeader {
    fn z_header(&self) -> u8;
}

pub trait ZLen: ZBodyLen {
    fn z_len(&self) -> usize;
}

pub trait ZEncode: ZBodyEncode {
    fn z_encode(&self, w: &mut ZWriter) -> ZCodecResult<()>;
}

pub trait ZDecode<'a>: Sized + ZBodyDecode<'a> {
    fn z_decode(r: &mut ZReader<'a>) -> ZCodecResult<Self>;
}

pub trait ZExtCount {
    fn z_ext_count(&self) -> usize;
}

macro_rules! derive_zstruct_with_body {
    (lt, $($ty:ty),*) => {
        $(
            impl<'a> $crate::ZLen for $ty {
                fn z_len(&self) -> usize {
                    <Self as $crate::ZBodyLen>::z_body_len(self)
                }
            }

            impl<'a> $crate::ZEncode for $ty {
                fn z_encode(&self, w: &mut $crate::ZWriter) -> $crate::ZCodecResult<()> {
                    <Self as $crate::ZBodyEncode>::z_body_encode(self, w)
                }
            }

            impl<'a> $crate::ZDecode<'a> for $ty {
                fn z_decode(r: &mut $crate::ZReader<'a>) -> $crate::ZCodecResult<Self> {
                    <Self as $crate::ZBodyDecode>::z_body_decode(r, ())
                }
            }
        )*
    };

    ($($ty:ty),*) => {
        $(
            impl $crate::ZLen for $ty {
                fn z_len(&self) -> usize {
                    <Self as $crate::ZBodyLen>::z_body_len(self)
                }
            }

            impl $crate::ZEncode for $ty {
                fn z_encode(&self, w: &mut $crate::ZWriter) -> $crate::ZCodecResult<()> {
                    <Self as $crate::ZBodyEncode>::z_body_encode(self, w)
                }
            }

            impl<'a> $crate::ZDecode<'a> for $ty {
                fn z_decode(r: &mut $crate::ZReader<'a>) -> $crate::ZCodecResult<Self> {
                    <Self as $crate::ZBodyDecode>::z_body_decode(r, ())
                }
            }
        )*
    };
}

pub(crate) use derive_zstruct_with_body;
