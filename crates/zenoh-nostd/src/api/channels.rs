use embassy_sync::{
    blocking_mutex::raw::NoopRawMutex,
    mutex::{Mutex, MutexGuard},
};
use embassy_time::Instant;
use heapless::{FnvIndexMap, Vec};
use higher_kinded_types::ForLt;
use zenoh_proto::keyexpr;

pub trait ZChannel<Value: ForLt> {
    type Item: for<'a> TryFrom<Value::Of<'a>, Error = crate::CollectionError>;

    fn send(
        &self,
        value: Value::Of<'_>,
    ) -> impl Future<Output = core::result::Result<(), crate::CollectionError>>;

    fn recv(&self) -> impl Future<Output = Self::Item>;
}

pub type HeaplessChannel<A, const QUEUED: usize> =
    embassy_sync::channel::Channel<NoopRawMutex, A, QUEUED>;

impl<Item, Value, const QUEUED: usize> ZChannel<Value> for HeaplessChannel<Item, QUEUED>
where
    Value: ForLt,
    Item: for<'a> TryFrom<Value::Of<'a>, Error = crate::CollectionError>,
{
    type Item = Item;

    async fn send(&self, value: Value::Of<'_>) -> core::result::Result<(), crate::CollectionError> {
        self.send(Self::Item::try_from(value)?).await;

        Ok(())
    }

    async fn recv(&self) -> Self::Item {
        self.receive().await
    }
}

pub trait ZChannels<Value: ForLt> {
    type Channel: ZChannel<Value>;

    type Guard<'a>
    where
        Self: 'a;

    fn new() -> Self;

    fn insert<'a>(
        &'a self,
        id: u32,
        ke: &'static keyexpr,
        timedout: Option<Instant>,
    ) -> impl Future<Output = core::result::Result<&'a Self::Channel, crate::CollectionError>>
    where
        Self::Channel: 'a,
        <Self::Channel as ZChannel<Value>>::Item: 'a;
    fn drop_timedout(&self) -> impl Future<Output = ()>;

    fn remove(
        &self,
        id: u32,
    ) -> impl Future<Output = core::result::Result<(), crate::CollectionError>>;

    fn lock(&self) -> impl Future<Output = Self::Guard<'_>>;

    fn get<'a, 'b>(
        &'a self,
        guard: &'b Self::Guard<'a>,
        id: u32,
    ) -> impl Future<Output = Option<&'b Self::Channel>>
    where
        Self::Channel: 'b,
        <Self::Channel as ZChannel<Value>>::Item: 'b;

    fn intersects<'r, 'a>(
        &'r self,
        guard: &'a Self::Guard<'r>,
        ke: &keyexpr,
    ) -> impl Future<Output = impl Iterator<Item = &'a Self::Channel>>
    where
        Self::Channel: 'a,
        <Self::Channel as ZChannel<Value>>::Item: 'a;
}

pub struct HeaplessChannelsInner<const QUEUED: usize, const CAPACITY: usize> {
    available: Vec<usize, CAPACITY>,

    keyexprs: FnvIndexMap<u32, &'static keyexpr, CAPACITY>,
    channels: FnvIndexMap<u32, u32, CAPACITY>,
    timedouts: FnvIndexMap<u32, Instant, CAPACITY>,
}

impl<const QUEUED: usize, const CAPACITY: usize> Default
    for HeaplessChannelsInner<QUEUED, CAPACITY>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<const QUEUED: usize, const CAPACITY: usize> HeaplessChannelsInner<QUEUED, CAPACITY> {
    pub fn new() -> Self {
        let mut available = Vec::new();

        for i in 0..CAPACITY {
            available.push(i).unwrap();
        }

        Self {
            available,
            keyexprs: FnvIndexMap::new(),
            channels: FnvIndexMap::new(),
            timedouts: FnvIndexMap::new(),
        }
    }

    pub fn drop_timedout(&mut self) {
        self.timedouts.retain(|id, timedout| {
            if Instant::now() >= *timedout {
                if let Some(index) = self.channels.remove(id) {
                    self.available.push(index as usize).ok();
                }

                self.keyexprs.remove(id);

                false
            } else {
                true
            }
        });
    }
}

pub struct HeaplessChannels<Item, const QUEUED: usize, const CAPACITY: usize> {
    channels: [HeaplessChannel<Item, QUEUED>; CAPACITY],
    inner: Mutex<NoopRawMutex, HeaplessChannelsInner<QUEUED, CAPACITY>>,
}

impl<Item, Value, const QUEUED: usize, const CAPACITY: usize> ZChannels<Value>
    for HeaplessChannels<Item, QUEUED, CAPACITY>
where
    Value: ForLt,
    Item: for<'a> TryFrom<Value::Of<'a>, Error = crate::CollectionError>,
{
    type Channel = HeaplessChannel<Item, QUEUED>;
    type Guard<'r>
        = MutexGuard<'r, NoopRawMutex, HeaplessChannelsInner<QUEUED, CAPACITY>>
    where
        Self: 'r;

    fn new() -> Self {
        Self {
            channels: core::array::from_fn(|_| HeaplessChannel::new()),
            inner: Mutex::new(HeaplessChannelsInner::new()),
        }
    }

    async fn get<'a, 'b>(&'a self, guard: &'b Self::Guard<'a>, id: u32) -> Option<&'b Self::Channel>
    where
        Self::Channel: 'b,
        <Self::Channel as ZChannel<Value>>::Item: 'b,
    {
        guard
            .channels
            .get(&id)
            .map(|ch| &self.channels[*ch as usize])
    }

    async fn insert<'r>(
        &'r self,
        id: u32,
        ke: &'static keyexpr,
        timedout: Option<Instant>,
    ) -> core::result::Result<&'r Self::Channel, crate::CollectionError>
    where
        Self::Channel: 'r,
    {
        let mut guard = self.inner.lock().await;

        if guard.channels.contains_key(&id) {
            return Err(crate::CollectionError::KeyAlreadyExists);
        }

        let index = guard
            .available
            .pop()
            .ok_or(crate::CollectionError::CollectionIsEmpty)?;

        guard
            .keyexprs
            .insert(id, ke)
            .map_err(|_| crate::CollectionError::CollectionIsFull)?;

        guard
            .channels
            .insert(id, index as u32)
            .map_err(|_| crate::CollectionError::CollectionIsFull)?;

        if let Some(timedout) = timedout {
            guard
                .timedouts
                .insert(id, timedout)
                .map_err(|_| crate::CollectionError::CollectionIsFull)?;
        }

        Ok(&self.channels[index])
    }

    async fn drop_timedout(&self) {
        self.inner.lock().await.drop_timedout();
    }

    async fn remove(&self, id: u32) -> core::result::Result<(), crate::CollectionError> {
        let mut guard = self.inner.lock().await;

        let index = guard
            .channels
            .remove(&id)
            .ok_or(crate::CollectionError::KeyNotFound)? as usize;

        guard.keyexprs.remove(&id);
        guard.timedouts.remove(&id);

        guard
            .available
            .push(index)
            .map_err(|_| crate::CollectionError::CollectionIsFull)?;

        Ok(())
    }

    async fn lock(&self) -> Self::Guard<'_> {
        self.inner.lock().await
    }

    async fn intersects<'r, 'a>(
        &'r self,
        guard: &'a Self::Guard<'r>,
        ke: &keyexpr,
    ) -> impl Iterator<Item = &'a Self::Channel>
    where
        Self::Channel: 'a,
        <Self::Channel as ZChannel<Value>>::Item: 'a,
    {
        guard.keyexprs.iter().filter_map(|(id, stored_ke)| {
            if stored_ke.intersects(ke) {
                let index = guard.channels.get(id)?;
                Some(&self.channels[*index as usize])
            } else {
                None
            }
        })
    }
}
