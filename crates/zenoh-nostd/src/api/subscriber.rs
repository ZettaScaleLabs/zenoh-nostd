use embassy_sync::channel::DynamicReceiver;
use zenoh_proto::{ZError, ZResult, keyexpr};

use crate::api::sample::ZOwnedSample;

pub mod callback;

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
            ZSubscriberInner::Sync => Err(ZError::CouldNotRecvFromChannel),
            ZSubscriberInner::Async(rx) => Ok(rx.receive().await),
        }
    }
}

#[macro_export]
macro_rules! zsubscriber {
    ($sync:expr) => {
        (
            $crate::ZSubscriberCallback::new_sync($sync),
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
            $crate::ZSubscriberCallback::new_async(channel),
            Some(channel.dyn_receiver()),
        )
    }};
}
