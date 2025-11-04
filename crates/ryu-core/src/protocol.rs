pub mod core;
pub use core::*;

pub mod network;
pub mod transport;
pub mod zenoh;

pub const MSG_ID_MASK: u8 = 0b0001_1111;

crate::__internal_err! {
    /// Errors related to IO operations on byte buffers
    #[err = "protocol error"]
    enum ProtocolError {
        CouldNotParse
    }
}

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

        impl ZStruct for $name<'_> {
            fn z_len(&self) -> usize {
                match self {
                    $(
                        Self::$variant(x) => <$variant as ZStruct>::z_len(x),
                    )*
                }
            }

            fn z_encode(&self, w: &mut ByteWriter) -> ByteIOResult<()> {
                match self {
                    $(
                        Self::$variant(x) => <$variant as ZStruct>::z_encode(x, w),
                    )*
                }
            }

            type ZType<'a> = $name<'a>;

            fn z_decode<'a>(r: &mut ByteReader<'a>) -> ByteIOResult<Self::ZType<'a>> {
                let id = r.peek_u8()? & MSG_ID_MASK;

                match id {
                    $(
                        <$variant>::ID => Ok(Self::ZType::$variant(<$variant as ZStruct>::z_decode(r)?)),
                    )*
                    _ => Err(crate::ByteIOError::CouldNotParse),
                }
            }
        }

        impl<'a> $name<'a> {
            #[cfg(test)]
            pub(crate) fn rand(zbuf: &mut ByteWriter<'a>) -> $name<'a> {
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
