use core::{
    future::poll_fn,
    task::{Context, Poll},
};

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use heapless::index_map::{FnvIndexMap, Iter};
use zenoh_proto::{ZError, ZResult, keyexpr, zbail};

use crate::{ZOwnedReply, ZReply};

pub trait ZRepliesAsyncCallback {
    fn poll_ready_to_send(&self, cx: &mut Context<'_>) -> Poll<()>;
    fn call(&self, reply: ZReply<'_>) -> ZResult<()>;
}

pub(crate) enum ZRepliesCallbackInner {
    Sync(fn(&ZReply<'_>) -> ()),
    Async(&'static dyn ZRepliesAsyncCallback),
}

pub struct ZRepliesCallback(ZRepliesCallbackInner);

impl ZRepliesCallback {
    pub fn new_sync(f: fn(&ZReply<'_>) -> ()) -> Self {
        Self(ZRepliesCallbackInner::Sync(f))
    }

    pub fn new_async(f: &'static dyn ZRepliesAsyncCallback) -> Self {
        Self(ZRepliesCallbackInner::Async(f))
    }

    pub(crate) fn is_async(&self) -> bool {
        matches!(self.0, ZRepliesCallbackInner::Async(_))
    }

    pub(crate) async fn call(&self, sample: ZReply<'_>) -> ZResult<()> {
        match self.0 {
            ZRepliesCallbackInner::Sync(f) => {
                f(&sample);
            }
            ZRepliesCallbackInner::Async(f) => {
                poll_fn(|cx| f.poll_ready_to_send(cx)).await;
                f.call(sample)?;
            }
        }

        Ok(())
    }
}

impl<const L: usize, const KE: usize, const PL: usize> ZRepliesAsyncCallback
    for Channel<CriticalSectionRawMutex, ZOwnedReply<KE, PL>, L>
{
    fn poll_ready_to_send(&self, cx: &mut Context<'_>) -> Poll<()> {
        self.poll_ready_to_send(cx)
    }

    fn call(&self, reply: ZReply<'_>) -> ZResult<()> {
        let reply = reply.into_owned()?;

        self.try_send(reply)
            .expect("You should have polled for readiness before calling the callback!");

        Ok(())
    }
}

pub trait ZRepliesCallbacks {
    fn insert(&mut self, id: u32, ke: &'static keyexpr, callback: ZRepliesCallback) -> ZResult<()>;
    fn intersects(&self, id: &u32, ke: &'_ keyexpr) -> bool;
    fn iter(&self) -> Iter<'_, u32, ZRepliesCallback>;

    fn remove(&mut self, id: &u32) -> Option<ZRepliesCallback>;
}

pub struct ZRepliesCallbackStorage<const N: usize> {
    lookup: FnvIndexMap<u32, &'static keyexpr, N>,
    callbacks: FnvIndexMap<u32, ZRepliesCallback, N>,
}

impl<const N: usize> Default for ZRepliesCallbackStorage<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> ZRepliesCallbackStorage<N> {
    pub fn new() -> Self {
        Self {
            lookup: FnvIndexMap::new(),
            callbacks: FnvIndexMap::new(),
        }
    }
}

impl<const N: usize> ZRepliesCallbacks for ZRepliesCallbackStorage<N> {
    fn insert(&mut self, id: u32, ke: &'static keyexpr, callback: ZRepliesCallback) -> ZResult<()> {
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

    fn iter(&self) -> Iter<'_, u32, ZRepliesCallback> {
        self.callbacks.iter()
    }

    fn remove(&mut self, id: &u32) -> Option<ZRepliesCallback> {
        self.lookup.remove(id);
        self.callbacks.remove(id)
    }
}
