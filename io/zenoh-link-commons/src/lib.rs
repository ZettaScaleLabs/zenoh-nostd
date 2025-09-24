#![no_std]

use zenoh_protocol::{
    core::{Locator, PriorityRange, Reliability},
    transport::BatchSize,
};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Link<const S: usize, const D: usize, const G: usize = 0> {
    pub src: Locator<S>,
    pub dst: Locator<D>,
    pub group: Option<Locator<G>>,
    pub mtu: BatchSize,
    pub is_streamed: bool,
    pub priorities: Option<PriorityRange>,
    pub reliability: Option<Reliability>,
}
