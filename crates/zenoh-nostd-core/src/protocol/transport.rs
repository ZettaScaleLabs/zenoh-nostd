use crate::ZExt;

#[cfg(test)]
use crate::{ZWriter, ZWriterExt};
#[cfg(test)]
use rand::{Rng, thread_rng};

pub mod close;
pub mod frame;
pub mod init;
pub mod keepalive;
pub mod open;

#[derive(ZExt, Debug, PartialEq)]
pub struct HasQoS {}

#[derive(ZExt, Debug, PartialEq)]
pub struct QoSLink {
    pub v: u64,
}

impl QoSLink {
    #[cfg(test)]
    pub(crate) fn rand(_: &mut ZWriter) -> Self {
        Self {
            v: thread_rng().r#gen(),
        }
    }
}

#[derive(ZExt, Debug, PartialEq)]
pub struct Auth<'a> {
    #[zenoh(size = remain)]
    pub payload: &'a [u8],
}

impl<'a> Auth<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut crate::ZWriter<'a>) -> Self {
        let payload = w
            .write_slot(thread_rng().gen_range(0..=64), |b: &mut [u8]| {
                thread_rng().fill(b);
                b.len()
            })
            .unwrap();

        Self { payload }
    }
}

#[derive(ZExt, Debug, PartialEq)]
pub struct MultiLink<'a> {
    #[zenoh(size = remain)]
    pub payload: &'a [u8],
}

impl<'a> MultiLink<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut crate::ZWriter<'a>) -> Self {
        let payload = w
            .write_slot(thread_rng().gen_range(0..=64), |b: &mut [u8]| {
                thread_rng().fill(b);
                b.len()
            })
            .unwrap();

        Self { payload }
    }
}

#[derive(ZExt, Debug, PartialEq)]
pub struct HasLowLatency {}

#[derive(ZExt, Debug, PartialEq)]
pub struct HasCompression {}

#[derive(ZExt, Debug, PartialEq)]
pub struct PatchType {
    pub int: u8,
}

impl PatchType {
    pub const NONE: Self = Self { int: 0 };
    pub const CURRENT: Self = Self { int: 1 };

    pub fn new(int: u8) -> Self {
        Self { int }
    }

    pub fn raw(self) -> u8 {
        self.int
    }

    pub fn has_fragmentation_markers(&self) -> bool {
        self.int >= 1
    }

    #[cfg(test)]
    pub(crate) fn rand(_: &mut ZWriter) -> Self {
        Self {
            int: thread_rng().r#gen(),
        }
    }
}
