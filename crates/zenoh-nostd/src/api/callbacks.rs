use embassy_time::Instant;
use heapless::FnvIndexMap;
use zenoh_proto::keyexpr;

use crate::api::{Callback, ZCallback};

pub trait ZCallbacks<A, B> {
    type Callback: ZCallback<A, B>;

    fn empty() -> Self;
    fn insert(
        &mut self,
        id: u32,
        ke: &'static keyexpr,
        timedout: Option<Instant>,
        callback: impl Into<Self::Callback>,
    ) -> core::result::Result<(), crate::CollectionError>;
    fn drop_timedout(&mut self);
    fn get<'r>(&'r self, id: u32) -> Option<&'r Self::Callback>;
    fn remove(&mut self, id: u32) -> core::result::Result<(), crate::CollectionError>;
    fn intersects<'r>(&'r self, ke: &keyexpr) -> impl Iterator<Item = &'r Self::Callback>
    where
        Self::Callback: 'r,
        A: 'r,
        B: 'r;
}

pub struct HeaplessCallbacks<A, B, const ASYNC_CALLBACK_MEMORY: usize, const CAPACITY: usize> {
    keyexprs: FnvIndexMap<u32, &'static keyexpr, CAPACITY>,
    callbacks: FnvIndexMap<u32, Callback<A, B, ASYNC_CALLBACK_MEMORY>, CAPACITY>,
    timedouts: FnvIndexMap<u32, Instant, CAPACITY>,
}

impl<A, B, const ASYNC_CALLBACK_MEMORY: usize, const CAPACITY: usize> ZCallbacks<A, B>
    for HeaplessCallbacks<A, B, ASYNC_CALLBACK_MEMORY, CAPACITY>
{
    type Callback = Callback<A, B, ASYNC_CALLBACK_MEMORY>;

    fn empty() -> Self {
        Self {
            keyexprs: FnvIndexMap::new(),
            callbacks: FnvIndexMap::new(),
            timedouts: FnvIndexMap::new(),
        }
    }

    fn insert(
        &mut self,
        id: u32,
        ke: &'static keyexpr,
        timedout: Option<Instant>,
        callback: impl Into<Self::Callback>,
    ) -> core::result::Result<(), crate::CollectionError> {
        if self.keyexprs.contains_key(&id) {
            return Err(crate::CollectionError::KeyAlreadyExists);
        }

        if self.callbacks.contains_key(&id) {
            return Err(crate::CollectionError::KeyAlreadyExists);
        }

        if self.timedouts.contains_key(&id) {
            return Err(crate::CollectionError::KeyAlreadyExists);
        }

        self.keyexprs
            .insert(id, ke)
            .map_err(|_| crate::CollectionError::CollectionIsFull)?;

        self.callbacks
            .insert(id, callback.into())
            .map_err(|_| crate::CollectionError::CollectionIsFull)?;

        if let Some(timedout) = timedout {
            self.timedouts
                .insert(id, timedout)
                .map_err(|_| crate::CollectionError::CollectionIsFull)?;
        }

        Ok(())
    }

    fn drop_timedout(&mut self) {
        self.timedouts.retain(|id, timedout| {
            if Instant::now() >= *timedout {
                self.keyexprs.remove(&id);
                self.callbacks.remove(&id);

                false
            } else {
                true
            }
        });
    }

    fn remove(&mut self, id: u32) -> core::result::Result<(), crate::CollectionError> {
        self.keyexprs.remove(&id);
        self.callbacks.remove(&id);
        self.timedouts.remove(&id);

        Ok(())
    }

    fn get<'r>(&'r self, id: u32) -> Option<&'r Self::Callback> {
        self.callbacks.get(&id)
    }

    fn intersects<'r>(&'r self, ke: &keyexpr) -> impl Iterator<Item = &'r Self::Callback>
    where
        Self::Callback: 'r,
    {
        self.keyexprs.iter().filter_map(|(id, registered_ke)| {
            if registered_ke.intersects(ke) {
                self.callbacks.get(id)
            } else {
                None
            }
        })
    }
}
