use embassy_time::Instant;
use heapless::{FnvIndexMap, IndexMapIter};
use zenoh_proto::{Error, ZResult, keyexpr, zbail};

use crate::ZReply;

pub struct ZRepliesCallback {
    callback: fn(&ZReply<'_>),
    expiration: Instant,
}

impl ZRepliesCallback {
    pub fn new(f: fn(&ZReply<'_>), expiration: Instant) -> Self {
        Self {
            callback: f,
            expiration,
        }
    }

    pub(crate) fn call(&self, reply: ZReply<'_>) {
        (self.callback)(&reply);
    }

    pub fn is_timed_out(&self) -> bool {
        Instant::now() >= self.expiration
    }
}

pub trait ZRepliesCallbacks {
    fn insert(
        &mut self,
        id: u32,
        ke: &'static keyexpr,
        callback: ZRepliesCallback,
    ) -> crate::ZResult<()>;
    fn intersects(&self, id: &u32, ke: &'_ keyexpr) -> bool;
    fn iter(&self) -> IndexMapIter<'_, u32, ZRepliesCallback>;

    fn remove(&mut self, id: &u32) -> Option<ZRepliesCallback>;

    fn drop_timedout(&mut self);
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
    fn insert(
        &mut self,
        id: u32,
        ke: &'static keyexpr,
        callback: ZRepliesCallback,
    ) -> crate::ZResult<()> {
        if self.lookup.contains_key(&id) {
            zbail!(Error::CallbackAlreadySet)
        }

        self.lookup
            .insert(id, ke)
            .map_err(|_| Error::CapacityExceeded)?;

        self.callbacks
            .insert(id, callback)
            .map_err(|_| Error::CapacityExceeded)
            .map(|_| ())
    }

    fn intersects(&self, id: &u32, ke: &'_ keyexpr) -> bool {
        if let Some(stored_ke) = self.lookup.get(id) {
            return stored_ke.intersects(ke);
        }

        false
    }

    fn iter(&self) -> IndexMapIter<'_, u32, ZRepliesCallback> {
        self.callbacks.iter()
    }

    fn remove(&mut self, id: &u32) -> Option<ZRepliesCallback> {
        self.lookup.remove(id);
        self.callbacks.remove(id)
    }

    fn drop_timedout(&mut self) {
        self.callbacks.retain(|id, callback| {
            if callback.is_timed_out() {
                crate::debug!("Dropping timed out reply callback for id {}", id);
                self.lookup.remove(id);
                false
            } else {
                true
            }
        });
    }
}
