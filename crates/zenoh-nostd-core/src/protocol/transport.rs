use crate::ZExt;

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

#[derive(ZExt, Debug, PartialEq)]
pub struct Auth<'a> {
    #[zenoh(size = remain)]
    pub payload: &'a [u8],
}

#[derive(ZExt, Debug, PartialEq)]
pub struct MultiLink<'a> {
    #[zenoh(size = remain)]
    pub payload: &'a [u8],
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
    pub fn rand() -> Self {
        Self {
            int: thread_rng().r#gen(),
        }
    }
}
