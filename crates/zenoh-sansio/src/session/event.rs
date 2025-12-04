use zenoh_proto::{
    ZResult,
    network::push::Push,
    transport::{init::InitSyn, open::OpenSyn},
};

#[derive(Debug, PartialEq)]
pub enum EventInner<'a> {
    None,
    InitSyn(InitSyn<'a>),
    OpenSyn(OpenSyn<'a>),
    KeepAlive,
    Close,

    Push(Push<'a>),
}

#[derive(Debug, PartialEq)]
pub struct Event<'a> {
    pub inner: EventInner<'a>,
}

impl Default for Event<'_> {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl Event<'_> {
    pub const EMPTY: Self = Self {
        inner: EventInner::None,
    };
}

pub trait EventAccumulator<'a> {
    fn push(&mut self, event: Event<'a>) -> crate::ZResult<(), ()>;
}

impl<'a> EventAccumulator<'a> for &mut [Event<'a>] {
    fn push(&mut self, event: Event<'a>) -> crate::ZResult<(), ()> {
        if self.is_empty() {
            Err(())
        } else {
            unsafe {
                *self.get_unchecked_mut(0) = event;
                *self = core::mem::take(self).get_unchecked_mut(1..);
            }
            Ok(())
        }
    }
}

pub trait AsEventAccumulator<'a> {
    type Acc<'s>: EventAccumulator<'a>
    where
        Self: 's,
        'a: 's;

    fn as_accumulator<'s>(&'s mut self) -> Self::Acc<'s>;
}

impl<'a, const N: usize> AsEventAccumulator<'a> for [Event<'a>; N] {
    type Acc<'s>
        = &'s mut [Event<'a>]
    where
        Self: 's;

    fn as_accumulator<'s>(&'s mut self) -> Self::Acc<'s> {
        self.as_mut_slice()
    }
}
