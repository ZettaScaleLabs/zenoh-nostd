use crate::{ZCodecResult, ZReader, ZWriter};

mod array;
mod bytes;
mod str;
mod uint;

/// Trait for getting the length of the body of a struct (without header if any)
pub trait ZBodyLen {
    fn z_body_len(&self) -> usize;
}

/// Trait for encoding the body of a struct (without header if any)
pub trait ZBodyEncode {
    fn z_body_encode(&self, w: &mut ZWriter) -> ZCodecResult<()>;
}

/// Trait for decoding the body of a struct, asking for a context (header) if needed;
pub trait ZBodyDecode<'a>: Sized {
    type Ctx;

    fn z_body_decode(r: &mut ZReader<'a>, ctx: Self::Ctx) -> ZCodecResult<Self>;
}

/// Trait for getting the header byte of a struct that has one
pub trait ZHeader {
    fn z_header(&self) -> u8;
}

/// Trait for getting the length of a struct (including header if any)
pub trait ZLen: ZBodyLen {
    fn z_len(&self) -> usize;
}

/// Trait for encoding a struct (first encoding header if any)
pub trait ZEncode: ZBodyEncode {
    fn z_encode(&self, w: &mut ZWriter) -> ZCodecResult<()>;
}

/// Trait for decoding a struct (first decoding the header if any)
pub trait ZDecode<'a>: Sized + ZBodyDecode<'a> {
    fn z_decode(r: &mut ZReader<'a>) -> ZCodecResult<Self>;
}

pub trait ZExtCount {
    fn z_ext_count(&self) -> usize;
}

#[derive(crate::ZStruct)]
#[zenoh(header = "Z|S:7")]
pub struct Test {
    a: u8,
}

/*
 *
 *
 *
 *
 *
 *
 *
 *
 *
 *
 *
 *
 *
 *
 *
 *
 *
 *
 *
 *
 *
 *
 *
 *
 *
 *
 *
 *
 */

// pub trait ZStructEncode {
//     fn z_len_without_header(&self) -> usize;

//     fn z_len(&self) -> usize {
//         let header_len = if <Self as ZStructEncode>::z_header(&self).is_some() {
//             1
//         } else {
//             0
//         };
//         header_len + <Self as ZStructEncode>::z_len_without_header(&self)
//     }

//     fn z_header(&self) -> Option<u8> {
//         None
//     }

//     fn z_encode_without_header(&self, w: &mut ZWriter) -> ZCodecResult<()>;

//     fn z_encode(&self, w: &mut ZWriter) -> ZCodecResult<()> {
//         if let Some(header) = <Self as ZStructEncode>::z_header(&self) {
//             <u8 as ZStructEncode>::z_encode(&header, w)?;
//         }

//         <Self as ZStructEncode>::z_encode_without_header(self, w)
//     }
// }

// pub trait ZStructDecode<'a> {
//     fn z_decode_with_header(r: &mut ZReader<'a>, h: u8) -> ZCodecResult<Self>
//     where
//         Self: Sized;

//     fn z_decode(r: &mut ZReader<'a>) -> ZCodecResult<Self>
//     where
//         Self: Sized,
//     {
//         <Self as ZStructDecode>::z_decode_with_header(r, 0) // Assumes no header for most types
//     }
// }
