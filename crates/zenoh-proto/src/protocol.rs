mod codec;

pub use codec::ZCodecError;
pub(crate) use codec::*;

mod core;
pub use core::*;

pub mod network;
pub mod transport;
pub mod zenoh;

mod endpoint;
pub use endpoint::*;

mod ke;
pub use ke::*;

crate::make_zerr! {
    /// Errors related to Zenoh protocol
    #[err = "zenoh protocol error"]
    enum ZProtocolError {
        NoProtocolSeparator,
        MetadataNotSupported,
        ConfigNotSupported,
        CouldNotParse
    }
}

macro_rules! aggregate_enum_full {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident <'a> {
            $(
                $variant:ident $(<$lt:lifetime>)?
            ),* $(,)?
        }
    ) => {
        $crate::aggregate_enum_batch! {
            $(#[$meta])*
            $vis enum $name<'a> {
                $(
                    $variant $(<$lt>)?,
                )*
            }
        }

        impl<'a> $crate::ZBodyDecode<'a> for $name<'a> {
            type Ctx = u8;

            fn z_body_decode(r: &mut $crate::ZReader<'a>, header: u8) -> $crate::ZCodecResult<Self> {
                let id = header & 0b0001_1111;

                match id {
                    $(
                        <$variant>::ID => Ok(Self::$variant(<$variant as $crate::ZBodyDecode>::z_body_decode(r, header)?)),
                    )*
                    _ => Err($crate::ZCodecError::CouldNotParse),
                }
            }
        }

        impl<'a> $crate::ZDecode<'a> for $name<'a> {
            fn z_decode(r: &mut $crate::ZReader<'a>) -> $crate::ZCodecResult<Self> {
                let header = <u8 as $crate::ZDecode>::z_decode(r)?;
                <Self as $crate::ZBodyDecode>::z_body_decode(r, header)
            }
        }
    };
}
pub(crate) use aggregate_enum_full;

macro_rules! aggregate_enum_batch {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident <'a> {
            $(
                $variant:ident $(<$lt:lifetime>)?
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis enum $name<'a> {
            $(
                $variant($variant$(<$lt>)?),
            )*
        }

        impl $crate::ZBodyLen for $name<'_> {
            fn z_body_len(&self) -> usize {
                match self {
                    $(
                        Self::$variant(x) => <$variant as $crate::ZBodyLen>::z_body_len(x),
                    )*
                }
            }
        }

        impl $crate::ZLen for $name<'_> {
            fn z_len(&self) -> usize {
                1 + <Self as $crate::ZBodyLen>::z_body_len(self)
            }
        }

        impl $crate::ZBodyEncode for $name<'_> {
            fn z_body_encode(&self, w: &mut $crate::ZWriter) -> $crate::ZCodecResult<()> {
                match self {
                    $(
                        Self::$variant(x) => <$variant as $crate::ZBodyEncode>::z_body_encode(x, w),
                    )*
                }
            }
        }

        impl $crate::ZEncode for $name<'_> {
            fn z_encode(&self, w: &mut $crate::ZWriter) -> $crate::ZCodecResult<()> {
                match self {
                    $(
                        Self::$variant(x) => <$variant as $crate::ZEncode>::z_encode(x, w),
                    )*
                }
            }
        }

        impl<'a> $name<'a> {
            #[cfg(test)]
            pub(crate) fn rand(zbuf: &mut $crate::ZWriter<'a>) -> $name<'a> {
                use rand::seq::SliceRandom;

                let mut rng = rand::thread_rng();
                let choices = [
                    $(
                        $variant::ID,
                    )*
                ];
                match *choices.choose(&mut rng).unwrap() {
                    $(
                        <$variant>::ID => Self::$variant(<$variant>::rand(zbuf)),
                    )*
                    _ => unreachable!(),
                }
            }
        }
    };
}

pub(crate) use aggregate_enum_batch;
