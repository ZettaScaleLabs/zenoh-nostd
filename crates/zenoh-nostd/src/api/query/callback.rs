use core::{
    future::poll_fn,
    task::{Context, Poll},
};

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use heapless::index_map::{FnvIndexMap, Iter};
use zenoh_proto::{ZError, ZResult, keyexpr, zbail};

use crate::{ZOwnedReply, ZReply};

pub trait ZQueryAsyncCallback {
    fn poll_ready_to_send(&self, cx: &mut Context<'_>) -> Poll<()>;
    fn call(&self, reply: ZReply<'_>) -> ZResult<()>;
}

pub(crate) enum ZQueryCallbackInner {
    Sync(fn(&ZReply<'_>) -> ()),
    Async(&'static dyn ZQueryAsyncCallback),
}

pub struct ZQueryCallback(ZQueryCallbackInner);

impl ZQueryCallback {
    pub fn new_sync(f: fn(&ZReply<'_>) -> ()) -> Self {
        Self(ZQueryCallbackInner::Sync(f))
    }

    pub fn new_async(f: &'static dyn ZQueryAsyncCallback) -> Self {
        Self(ZQueryCallbackInner::Async(f))
    }

    pub(crate) fn is_async(&self) -> bool {
        matches!(self.0, ZQueryCallbackInner::Async(_))
    }

    pub(crate) async fn call(&self, sample: ZReply<'_>) -> ZResult<()> {
        match self.0 {
            ZQueryCallbackInner::Sync(f) => {
                f(&sample);
            }
            ZQueryCallbackInner::Async(f) => {
                poll_fn(|cx| f.poll_ready_to_send(cx)).await;
                f.call(sample)?;
            }
        }

        Ok(())
    }
}

impl<const L: usize, const KE: usize, const PL: usize> ZQueryAsyncCallback
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

pub trait ZQueryCallbacks {
    fn insert(&mut self, id: u32, ke: &'static keyexpr, callback: ZQueryCallback) -> ZResult<()>;
    fn intersects(&self, id: &u32, ke: &'_ keyexpr) -> bool;
    fn iter(&self) -> Iter<'_, u32, ZQueryCallback>;

    fn remove(&mut self, id: &u32) -> Option<ZQueryCallback>;
}

pub struct ZQueryCallbackStorage<const N: usize> {
    lookup: FnvIndexMap<u32, &'static keyexpr, N>,
    callbacks: FnvIndexMap<u32, ZQueryCallback, N>,
}

impl<const N: usize> Default for ZQueryCallbackStorage<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> ZQueryCallbackStorage<N> {
    pub fn new() -> Self {
        Self {
            lookup: FnvIndexMap::new(),
            callbacks: FnvIndexMap::new(),
        }
    }
}

impl<const N: usize> ZQueryCallbacks for ZQueryCallbackStorage<N> {
    fn insert(&mut self, id: u32, ke: &'static keyexpr, callback: ZQueryCallback) -> ZResult<()> {
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

    fn iter(&self) -> Iter<'_, u32, ZQueryCallback> {
        self.callbacks.iter()
    }

    fn remove(&mut self, id: &u32) -> Option<ZQueryCallback> {
        self.lookup.remove(id);
        self.callbacks.remove(id)
    }
}
