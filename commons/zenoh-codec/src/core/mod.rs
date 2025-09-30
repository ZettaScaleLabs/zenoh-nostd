// use heapless::{String, Vec};
use zenoh_result::{zbail, ZResult, ZE};

use crate::{RCodec, WCodec, Writer, Zenoh080};

const VLE_LEN_MAX: usize = vle_len(u64::MAX);

const fn vle_len(x: u64) -> usize {
    const B1: u64 = u64::MAX << 7;
    const B2: u64 = u64::MAX << (7 * 2);
    const B3: u64 = u64::MAX << (7 * 3);
    const B4: u64 = u64::MAX << (7 * 4);
    const B5: u64 = u64::MAX << (7 * 5);
    const B6: u64 = u64::MAX << (7 * 6);
    const B7: u64 = u64::MAX << (7 * 7);
    const B8: u64 = u64::MAX << (7 * 8);

    if (x & B1) == 0 {
        1
    } else if (x & B2) == 0 {
        2
    } else if (x & B3) == 0 {
        3
    } else if (x & B4) == 0 {
        4
    } else if (x & B5) == 0 {
        5
    } else if (x & B6) == 0 {
        6
    } else if (x & B7) == 0 {
        7
    } else if (x & B8) == 0 {
        8
    } else {
        9
    }
}

impl WCodec<u8> for Zenoh080 {
    fn write(&self, message: &u8, mut support: &mut [u8]) -> ZResult<usize> {
        support.write_u8(*message)
    }
}

impl RCodec<u8> for Zenoh080 {
    fn read(&self, support: &[u8]) -> ZResult<(u8, usize)> {
        if support.is_empty() {
            zbail!(ZE::CapacityExceeded);
        }

        Ok((support[0], 1))
    }
}

impl WCodec<u64> for Zenoh080 {
    fn write(&self, message: &u64, mut support: &mut [u8]) -> ZResult<usize> {
        let len = vle_len(*message);
        if support.len() < len {
            zbail!(ZE::CapacityExceeded);
        }

        let mut value = *message;
        for i in 0..len {
            let byte = (value & 0x7F) as u8;
            value >>= 7;
            if i < len - 1 {
                support.write_u8(byte | 0x80)?;
            } else {
                support.write_u8(byte)?;
            }
        }

        Ok(len)
    }
}

impl RCodec<u64> for Zenoh080 {
    fn read(&self, support: &[u8]) -> ZResult<(u64, usize)> {
        let mut value: u64 = 0;
        let mut shift = 0;
        let mut len = 0;

        for byte in support {
            let byte = *byte;
            value |= ((byte & 0x7F) as u64) << shift;
            len += 1;

            if (byte & 0x80) == 0 {
                return Ok((value, len));
            }

            shift += 7;
            if shift >= 64 || len >= VLE_LEN_MAX {
                zbail!(ZE::ReadFailure);
            }
        }

        zbail!(ZE::CapacityExceeded);
    }
}

// impl<const N: usize> WCodec<[u8; N]> for Zenoh080 {
//     fn write(&self, message: &[u8; N], mut support: &mut [u8]) -> ZResult<usize> {
//         support.write(message, message.len())
//     }
// }

// impl<const N: usize> RCodec<[u8; N]> for Zenoh080 {
//     fn read(&self, support: &[u8]) -> ZResult<([u8; N], usize)> {
//         if support.len() < N {
//             zbail!(ZE::CapacityExceeded);
//         }

//         let mut array = [0u8; N];
//         array.copy_from_slice(&support[..N]);

//         Ok((array, N))
//     }
// }

// impl WCodec<&[u8]> for Zenoh080 {
//     fn write(&self, message: &&[u8], mut support: &mut [u8]) -> ZResult<usize> {
//         support.write(message, message.len())
//     }
// }

// impl<const N: usize> WCodec<Vec<u8, N>> for Zenoh080 {
//     fn write(&self, message: &Vec<u8, N>, mut support: &mut [u8]) -> ZResult<usize> {
//         support.write(message.as_slice(), message.len())
//     }
// }

// impl WCodec<&str> for Zenoh080 {
//     fn write(&self, message: &&str, mut support: &mut [u8]) -> ZResult<usize> {
//         support.write(message.as_bytes(), message.len())
//     }
// }

// impl<const N: usize> WCodec<&String<N>> for Zenoh080 {
//     fn write(&self, message: &&String<N>, mut support: &mut [u8]) -> ZResult<usize> {
//         support.write(message.as_bytes(), message.len())
//     }
// }
