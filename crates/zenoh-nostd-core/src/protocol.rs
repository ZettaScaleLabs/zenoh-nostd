pub mod core;
pub use core::*;

// pub mod network;
// pub mod transport;
pub mod zenoh;

/// Macro to define an aggregate ZStruct enum
#[macro_export]
macro_rules! __internal_zaggregate {
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

        impl $crate::ZStructEncode for $name<'_> {
            fn z_len(&self) -> usize {
                match self {
                    $(
                        Self::$variant(x) => <$variant as $crate::ZStructEncode>::z_len(x),
                    )*
                }
            }

            fn z_encode(&self, w: &mut $crate::ZWriter) -> $crate::ZCodecResult<()> {
                match self {
                    $(
                        Self::$variant(x) => <$variant as $crate::ZStructEncode>::z_encode(x, w),
                    )*
                }
            }
        }

        impl<'a> $crate::ZStructDecode<'a> for $name<'a> {
            fn z_decode(r: &mut $crate::ZReader<'a>) -> $crate::ZCodecResult<Self> {
                let id = <$crate::ZReader as $crate::ZReaderExt>::peek_u8(r)? & 0b0001_1111;

                match id {
                    $(
                        <$variant>::ID => Ok(Self::$variant(<$variant as $crate::ZStructDecode>::z_decode(r)?)),
                    )*
                    _ => Err($crate::ZCodecError::CouldNotParse),
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
