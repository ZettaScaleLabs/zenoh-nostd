use core::task::{Context, Poll};

use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    channel::{Channel, DynamicReceiver},
};
use heapless::{FnvIndexMap, IndexMapIter};
use zenoh_proto::{ZError, ZResult, keyexpr, zbail};

use crate::{ZOwnedQuery, ZQuery, platform::Platform};

pub struct ZQueryable<
    T: Platform + 'static,
    const MAX_KEYEXPR: usize,
    const MAX_PARAMETERS: usize,
    const MAX_PAYLOAD: usize,
> {
    ke: &'static keyexpr,
    inner: DynamicReceiver<'static, ZOwnedQuery<T, MAX_KEYEXPR, MAX_PARAMETERS, MAX_PAYLOAD>>,
}

impl<
    T: Platform + 'static,
    const MAX_KEYEXPR: usize,
    const MAX_PARAMETERS: usize,
    const MAX_PAYLOAD: usize,
> ZQueryable<T, MAX_KEYEXPR, MAX_PARAMETERS, MAX_PAYLOAD>
{
    pub(crate) fn new(
        ke: &'static keyexpr,
        rx: DynamicReceiver<'static, ZOwnedQuery<T, MAX_KEYEXPR, MAX_PARAMETERS, MAX_PAYLOAD>>,
    ) -> Self {
        Self { ke, inner: rx }
    }

    pub fn keyexpr(&self) -> &'static keyexpr {
        self.ke
    }

    pub async fn recv(
        &self,
    ) -> zenoh_proto::ZResult<ZOwnedQuery<T, MAX_KEYEXPR, MAX_PARAMETERS, MAX_PAYLOAD>> {
        Ok(self.inner.receive().await)
    }
}

pub trait ZQueryableAsyncCallback<T: Platform + 'static> {
    fn poll_ready_to_send(&self, cx: &mut core::task::Context<'_>) -> core::task::Poll<()>;
    fn call(&self, query: ZQuery<T>) -> zenoh_proto::ZResult<()>;
}

pub struct ZQueryableCallback<T: Platform + 'static>(&'static dyn ZQueryableAsyncCallback<T>);

impl<T: Platform + 'static> ZQueryableCallback<T> {
    pub fn new(f: &'static dyn ZQueryableAsyncCallback<T>) -> Self {
        Self(f)
    }

    pub(crate) async fn call(&self, query: ZQuery<'_, T>) -> zenoh_proto::ZResult<()> {
        use core::future::poll_fn;

        poll_fn(|cx| self.0.poll_ready_to_send(cx)).await;
        self.0.call(query)?;

        Ok(())
    }
}

impl<T: Platform + 'static, const L: usize, const KE: usize, const PM: usize, const PL: usize>
    ZQueryableAsyncCallback<T> for Channel<CriticalSectionRawMutex, ZOwnedQuery<T, KE, PM, PL>, L>
{
    fn poll_ready_to_send(&self, cx: &mut Context<'_>) -> Poll<()> {
        self.poll_ready_to_send(cx)
    }

    fn call(&self, query: ZQuery<'_, T>) -> ZResult<()> {
        let query = query.into_owned()?;

        match self.try_send(query) {
            Ok(()) => {}
            Err(_) => panic!("You should have called poll before"),
        }

        Ok(())
    }
}

pub trait ZQueryableCallbacks<T: Platform + 'static> {
    fn insert(
        &mut self,
        id: u32,
        ke: &'static keyexpr,
        callback: ZQueryableCallback<T>,
    ) -> ZResult<()>;
    fn intersects(&self, id: &u32, ke: &'_ keyexpr) -> bool;
    fn iter(&self) -> IndexMapIter<'_, u32, ZQueryableCallback<T>>;
}

pub struct ZQueryableCallbackStorage<T: Platform + 'static, const N: usize> {
    lookup: FnvIndexMap<u32, &'static keyexpr, N>,
    callbacks: FnvIndexMap<u32, ZQueryableCallback<T>, N>,
}

impl<T: Platform + 'static, const N: usize> Default for ZQueryableCallbackStorage<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Platform + 'static, const N: usize> ZQueryableCallbackStorage<T, N> {
    pub fn new() -> Self {
        Self {
            lookup: FnvIndexMap::new(),
            callbacks: FnvIndexMap::new(),
        }
    }
}

impl<T: Platform + 'static, const N: usize> ZQueryableCallbacks<T>
    for ZQueryableCallbackStorage<T, N>
{
    fn insert(
        &mut self,
        id: u32,
        ke: &'static keyexpr,
        callback: ZQueryableCallback<T>,
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

    fn iter(&self) -> IndexMapIter<'_, u32, ZQueryableCallback<T>> {
        self.callbacks.iter()
    }
}

#[macro_export]
macro_rules! zqueryable {
    ($platform:ident, QUEUE_SIZE: $queue:expr, MAX_KEYEXPR: $ke:expr, MAX_PARAMS: $pm:expr, MAX_PAYLOAD: $pl:expr) => {{
        static CHANNEL: static_cell::StaticCell<
            embassy_sync::channel::Channel<
                embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
                $crate::ZOwnedQuery<$platform, $ke, $pm, $pl>,
                $queue,
            >,
        > = static_cell::StaticCell::new();

        let channel = CHANNEL.init(embassy_sync::channel::Channel::new());

        (
            $crate::ZQueryableCallback::new(channel),
            channel.dyn_receiver(),
        )
    }};
}
