use embassy_sync::channel::DynamicReceiver;
use heapless::index_map::{FnvIndexMap, Iter};
use zenoh_proto::{ZError, ZResult, keyexpr, zbail};

use crate::api::{callback::ZCallback, sample::ZOwnedSample};

pub enum ZSubscriberInner<const MAX_KEYEXPR: usize, const MAX_PAYLOAD: usize> {
    Sync,
    Async(DynamicReceiver<'static, ZOwnedSample<MAX_KEYEXPR, MAX_PAYLOAD>>),
}

pub struct ZSubscriber<const MAX_KEYEXPR: usize, const MAX_PAYLOAD: usize> {
    id: u32,
    ke: &'static keyexpr,
    inner: ZSubscriberInner<MAX_KEYEXPR, MAX_PAYLOAD>,
}

impl<const MAX_KEYEXPR: usize, const MAX_PAYLOAD: usize> ZSubscriber<MAX_KEYEXPR, MAX_PAYLOAD> {
    pub(crate) fn new_sync(id: u32, ke: &'static keyexpr) -> Self {
        Self {
            id,
            ke,
            inner: ZSubscriberInner::Sync,
        }
    }

    pub(crate) fn new_async(
        id: u32,
        ke: &'static keyexpr,
        rx: DynamicReceiver<'static, ZOwnedSample<MAX_KEYEXPR, MAX_PAYLOAD>>,
    ) -> Self {
        ZSubscriber {
            id,
            ke,
            inner: ZSubscriberInner::Async(rx),
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn keyexpr(&self) -> &'static keyexpr {
        self.ke
    }

    pub async fn recv(&self) -> ZResult<ZOwnedSample<MAX_KEYEXPR, MAX_PAYLOAD>> {
        match &self.inner {
            ZSubscriberInner::Sync => Err(ZError::CouldNotRecvFromSubscriber),
            ZSubscriberInner::Async(rx) => Ok(rx.receive().await),
        }
    }
}

pub trait ZSubscriberCallbacks {
    fn insert(&mut self, id: u32, ke: &'static keyexpr, callback: ZCallback) -> ZResult<()>;
    fn intersects(&self, id: &u32, ke: &'_ keyexpr) -> bool;
    fn iter(&self) -> Iter<'_, u32, ZCallback>;
}

pub struct ZSubscriberCallbackStorage<const N: usize> {
    lookup: FnvIndexMap<u32, &'static keyexpr, N>,
    callbacks: FnvIndexMap<u32, ZCallback, N>,
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
    fn insert(&mut self, id: u32, ke: &'static keyexpr, callback: ZCallback) -> ZResult<()> {
        if self.lookup.contains_key(&id) {
            zbail!(ZError::SubscriberCallbackAlreadySet)
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

    fn iter(&self) -> Iter<'_, u32, ZCallback> {
        self.callbacks.iter()
    }
}

#[macro_export]
macro_rules! zsubscriber {
    ($sync:expr) => {
        (
            $crate::ZCallback::new_sync($sync),
            None::<embassy_sync::channel::DynamicReceiver<'static, $crate::ZOwnedSample<0, 0>>>,
        )
    };

    (QUEUE_SIZE: $queue:expr, MAX_KEYEXPR: $ke:expr, MAX_PAYLOAD: $pl:expr) => {{
        static CHANNEL: static_cell::StaticCell<
            embassy_sync::channel::Channel<
                embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
                $crate::ZOwnedSample<$ke, $pl>,
                $queue,
            >,
        > = static_cell::StaticCell::new();

        let channel = CHANNEL.init(embassy_sync::channel::Channel::new());

        (
            $crate::ZCallback::new_async(channel),
            Some(channel.dyn_receiver()),
        )
    }};
}
