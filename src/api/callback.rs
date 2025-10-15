use core::{
    future::poll_fn,
    task::{Context, Poll},
};

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};

use crate::{
    api::sample::{ZOwnedSample, ZSample},
    result::ZResult,
};

pub trait ZAsyncCallback {
    fn poll_ready_to_send(&self, cx: &mut Context<'_>) -> Poll<()>;
    fn call(&self, sample: ZSample<'_>) -> ZResult<()>;
}

pub(crate) enum ZCallbackInner {
    Sync(fn(&ZSample<'_>) -> ()),
    Async(&'static dyn ZAsyncCallback),
}

pub struct ZCallback(ZCallbackInner);

impl ZCallback {
    pub fn new_sync(f: fn(&ZSample<'_>) -> ()) -> Self {
        Self(ZCallbackInner::Sync(f))
    }

    pub fn new_async(f: &'static dyn ZAsyncCallback) -> Self {
        Self(ZCallbackInner::Async(f))
    }

    pub(crate) fn is_async(&self) -> bool {
        matches!(self.0, ZCallbackInner::Async(_))
    }

    pub(crate) async fn call(&self, sample: ZSample<'_>) -> ZResult<()> {
        match self.0 {
            ZCallbackInner::Sync(f) => {
                f(&sample);
            }
            ZCallbackInner::Async(f) => {
                poll_fn(|cx| f.poll_ready_to_send(cx)).await;
                f.call(sample)?;
            }
        }

        Ok(())
    }
}

impl<const L: usize, const KE: usize, const PL: usize> ZAsyncCallback
    for Channel<CriticalSectionRawMutex, ZOwnedSample<KE, PL>, L>
{
    fn poll_ready_to_send(&self, cx: &mut Context<'_>) -> Poll<()> {
        self.poll_ready_to_send(cx)
    }

    fn call(&self, sample: ZSample<'_>) -> ZResult<()> {
        let sample = sample.into_owned()?;

        self.try_send(sample)
            .expect("You should have polled for readiness before calling the callback!");

        Ok(())
    }
}
