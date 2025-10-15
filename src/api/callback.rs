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

pub enum ZCallback {
    Sync(fn(&ZSample<'_>) -> ()),
    Async(&'static dyn ZAsyncCallback),
}

impl ZCallback {
    pub fn is_async(&self) -> bool {
        matches!(self, ZCallback::Async(_))
    }

    pub async fn call(&self, sample: ZSample<'_>) -> ZResult<()> {
        match self {
            ZCallback::Sync(f) => {
                f(&sample);
            }
            ZCallback::Async(f) => {
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
