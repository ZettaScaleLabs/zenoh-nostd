use embassy_sync::channel::DynamicReceiver;
use zenoh_proto::keyexpr;

use crate::ZOwnedReply;

pub mod callback;

pub enum ZQueryInner<const MAX_KEYEXPR: usize, const MAX_PAYLOAD: usize> {
    Sync,
    Async(DynamicReceiver<'static, ZOwnedReply<MAX_KEYEXPR, MAX_PAYLOAD>>),
}

pub struct ZQuery<const MAX_KEYEXPR: usize, const MAX_PAYLOAD: usize> {
    id: u32,
    ke: &'static keyexpr,
    inner: ZQueryInner<MAX_KEYEXPR, MAX_PAYLOAD>,
}

impl<const MAX_KEYEXPR: usize, const MAX_PAYLOAD: usize> ZQuery<MAX_KEYEXPR, MAX_PAYLOAD> {
    pub(crate) fn new_sync(id: u32, ke: &'static keyexpr) -> Self {
        Self {
            id,
            ke,
            inner: ZQueryInner::Sync,
        }
    }

    pub(crate) fn new_async(
        id: u32,
        ke: &'static keyexpr,
        rx: DynamicReceiver<'static, ZOwnedReply<MAX_KEYEXPR, MAX_PAYLOAD>>,
    ) -> Self {
        ZQuery {
            id,
            ke,
            inner: ZQueryInner::Async(rx),
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn keyexpr(&self) -> &'static keyexpr {
        self.ke
    }

    pub async fn recv(&self) -> crate::ZResult<ZOwnedReply<MAX_KEYEXPR, MAX_PAYLOAD>> {
        match &self.inner {
            ZQueryInner::Sync => Err(zenoh_proto::ZError::CouldNotRecvFromChannel),
            ZQueryInner::Async(rx) => Ok(rx.receive().await),
        }
    }
}

#[macro_export]
macro_rules! zquery {
    ($sync:expr) => {
        (
            $crate::ZQueryCallback::new_sync($sync),
            None::<embassy_sync::channel::DynamicReceiver<'static, $crate::ZOwnedReply<0, 0>>>,
        )
    };

    (QUEUE_SIZE: $queue:expr, MAX_KEYEXPR: $ke:expr, MAX_PAYLOAD: $pl:expr) => {{
        static CHANNEL: static_cell::StaticCell<
            embassy_sync::channel::Channel<
                embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
                $crate::ZOwnedReply<$ke, $pl>,
                $queue,
            >,
        > = static_cell::StaticCell::new();

        let channel = CHANNEL.init(embassy_sync::channel::Channel::new());

        (
            $crate::ZQueryCallback::new_async(channel),
            Some(channel.dyn_receiver()),
        )
    }};
}
