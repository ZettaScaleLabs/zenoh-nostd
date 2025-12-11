use elain::{Align, Alignment};
use embassy_time::Instant;
use heapless::FnvIndexMap;
use zenoh_proto::keyexpr;

use crate::api::{AsyncCallback, ZCallback};

pub trait ZCallbacks<Arg, Ret> {
    type Callback: ZCallback<Arg, Ret>;

    fn empty() -> Self;
    fn insert(
        &mut self,
        id: u32,
        ke: &'static keyexpr,
        timedout: Option<Instant>,
        callback: Self::Callback,
    ) -> core::result::Result<(), crate::CollectionError>;
    fn drop_timedout(&mut self);
    fn get<'r>(&'r self, id: u32) -> Option<&'r Self::Callback>;
    fn remove(&mut self, id: u32) -> core::result::Result<(), crate::CollectionError>;
    fn intersects<'r>(&'r self, ke: &keyexpr) -> impl Iterator<Item = &'r Self::Callback>
    where
        Self::Callback: 'r,
        Arg: 'r,
        Ret: 'r;
}

pub struct HeaplessCallbacks<
    Arg,
    Ret,
    const CAPACITY: usize,
    const CALLBACK_SIZE: usize = { size_of::<usize>() },
    const CALLBACK_ALIGN: usize = { size_of::<usize>() },
    const FUTURE_SIZE: usize = { 4 * size_of::<usize>() },
    const FUTURE_ALIGN: usize = { size_of::<usize>() },
> where
    Align<CALLBACK_ALIGN>: Alignment,
    Align<FUTURE_ALIGN>: Alignment,
{
    keyexprs: FnvIndexMap<u32, &'static keyexpr, CAPACITY>,
    callbacks: FnvIndexMap<
        u32,
        AsyncCallback<Arg, Ret, CALLBACK_SIZE, CALLBACK_ALIGN, FUTURE_SIZE, FUTURE_ALIGN>,
        CAPACITY,
    >,
    timedouts: FnvIndexMap<u32, Instant, CAPACITY>,
}

impl<
    Arg,
    Ret,
    const CAPACITY: usize,
    const CALLBACK_SIZE: usize,
    const CALLBACK_ALIGN: usize,
    const FUTURE_SIZE: usize,
    const FUTURE_ALIGN: usize,
> ZCallbacks<Arg, Ret>
    for HeaplessCallbacks<
        Arg,
        Ret,
        CAPACITY,
        CALLBACK_SIZE,
        CALLBACK_ALIGN,
        FUTURE_SIZE,
        FUTURE_ALIGN,
    >
where
    Align<CALLBACK_ALIGN>: Alignment,
    Align<FUTURE_ALIGN>: Alignment,
{
    type Callback =
        AsyncCallback<Arg, Ret, CALLBACK_SIZE, CALLBACK_ALIGN, FUTURE_SIZE, FUTURE_ALIGN>;

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
        callback: Self::Callback,
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
            .insert(id, callback)
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
