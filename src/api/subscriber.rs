use embassy_sync::channel::DynamicReceiver;
use heapless::index_map::{FnvIndexMap, Iter};

use crate::{
    api::{callback::ZCallback, sample::ZOwnedSample},
    keyexpr::borrowed::keyexpr,
    result::{ZError, ZResult},
    zbail,
};

pub enum ZSubscriberInner<const KE: usize, const PL: usize> {
    Sync,
    Async(DynamicReceiver<'static, ZOwnedSample<KE, PL>>),
}

pub struct ZSubscriber<const KE: usize, const PL: usize> {
    id: u32,
    ke: &'static keyexpr,
    inner: ZSubscriberInner<KE, PL>,
}

impl<const KE: usize, const PL: usize> ZSubscriber<KE, PL> {
    pub fn sync_sub(id: u32, ke: &'static keyexpr) -> Self {
        Self {
            id,
            ke,
            inner: ZSubscriberInner::Sync,
        }
    }

    pub fn async_sub(
        id: u32,
        ke: &'static keyexpr,
        rx: DynamicReceiver<'static, ZOwnedSample<KE, PL>>,
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

    pub async fn recv(&self) -> ZResult<ZOwnedSample<KE, PL>> {
        match &self.inner {
            ZSubscriberInner::Sync => Err(ZError::Invalid),
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
            zbail!(ZError::Invalid)
        }

        self.lookup.insert(id, ke).map_err(|_| ZError::Invalid)?;

        self.callbacks
            .insert(id, callback)
            .map_err(|_| ZError::Invalid)
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
            $crate::api::callback::ZCallback::Sync($sync),
            None::<
                embassy_sync::channel::DynamicReceiver<
                    'static,
                    $crate::api::sample::ZOwnedSample<0, 0>,
                >,
            >,
        )
    };

    (QUEUE: $queue:expr, KE: $ke:expr, PL: $pl:expr) => {{
        static CHANNEL: static_cell::StaticCell<
            embassy_sync::channel::Channel<
                embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
                $crate::api::sample::ZOwnedSample<$ke, $pl>,
                $queue,
            >,
        > = static_cell::StaticCell::new();

        let channel = CHANNEL.init(embassy_sync::channel::Channel::new());

        (
            $crate::api::callback::ZCallback::Async(channel),
            Some(channel.dyn_receiver()),
        )
    }};
}
