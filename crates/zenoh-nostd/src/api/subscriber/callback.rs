use core::{
    future::poll_fn,
    task::{Context, Poll},
};

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use heapless::index_map::{FnvIndexMap, Iter};
use zenoh_proto::{ZError, ZResult, keyexpr, zbail};

use crate::api::sample::{ZOwnedSample, ZSample};

pub trait ZSubscriberAsyncCallback {
    fn poll_ready_to_send(&self, cx: &mut Context<'_>) -> Poll<()>;
    fn call(&self, sample: ZSample<'_>) -> ZResult<()>;
}

pub(crate) enum ZSubscriberCallbackInner {
    Sync(fn(&ZSample<'_>) -> ()),
    Async(&'static dyn ZSubscriberAsyncCallback),
}

pub struct ZSubscriberCallback(ZSubscriberCallbackInner);

impl ZSubscriberCallback {
    pub fn new_sync(f: fn(&ZSample<'_>) -> ()) -> Self {
        Self(ZSubscriberCallbackInner::Sync(f))
    }

    pub fn new_async(f: &'static dyn ZSubscriberAsyncCallback) -> Self {
        Self(ZSubscriberCallbackInner::Async(f))
    }

    pub(crate) fn is_async(&self) -> bool {
        matches!(self.0, ZSubscriberCallbackInner::Async(_))
    }

    pub(crate) async fn call(&self, sample: ZSample<'_>) -> ZResult<()> {
        match self.0 {
            ZSubscriberCallbackInner::Sync(f) => {
                f(&sample);
            }
            ZSubscriberCallbackInner::Async(f) => {
                poll_fn(|cx| f.poll_ready_to_send(cx)).await;
                f.call(sample)?;
            }
        }

        Ok(())
    }
}

impl<const L: usize, const KE: usize, const PL: usize> ZSubscriberAsyncCallback
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

pub trait ZSubscriberCallbacks {
    fn insert(
        &mut self,
        id: u32,
        ke: &'static keyexpr,
        callback: ZSubscriberCallback,
    ) -> ZResult<()>;
    fn intersects(&self, id: &u32, ke: &'_ keyexpr) -> bool;
    fn iter(&self) -> Iter<'_, u32, ZSubscriberCallback>;
}

pub struct ZSubscriberCallbackStorage<const N: usize> {
    lookup: FnvIndexMap<u32, &'static keyexpr, N>,
    callbacks: FnvIndexMap<u32, ZSubscriberCallback, N>,
}

impl<const N: usize> Default for ZSubscriberCallbackStorage<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> ZSubscriberCallbackStorage<N> {
    pub fn new() -> Self {
        Self {
            lookup: FnvIndexMap::new(),
            callbacks: FnvIndexMap::new(),
        }
    }
}

impl<const N: usize> ZSubscriberCallbacks for ZSubscriberCallbackStorage<N> {
    fn insert(
        &mut self,
        id: u32,
        ke: &'static keyexpr,
        callback: ZSubscriberCallback,
    ) -> ZResult<()> {
        if self.lookup.contains_key(&id) {
            zbail!(ZError::CallbackAlreadySet)
        }

        self.lookup
            .insert(id, ke)
            .map_err(|_| ZError::CapacityExceeded)?;

        self.callbacks
            .insert(id, callback)
            .map_err(|_| ZError::CapacityExceeded)
            .map(|_| ())
    }

    fn intersects(&self, id: &u32, ke: &'_ keyexpr) -> bool {
        if let Some(stored_ke) = self.lookup.get(id) {
            return stored_ke.intersects(ke);
        }

        false
    }

    fn iter(&self) -> Iter<'_, u32, ZSubscriberCallback> {
        self.callbacks.iter()
    }
}
