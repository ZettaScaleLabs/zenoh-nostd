use embassy_futures::select::select;
use embassy_sync::channel::DynamicReceiver;
use embassy_time::Duration;
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
    timeout: Duration,
    inner: ZQueryInner<MAX_KEYEXPR, MAX_PAYLOAD>,
}

impl<const MAX_KEYEXPR: usize, const MAX_PAYLOAD: usize> ZQuery<MAX_KEYEXPR, MAX_PAYLOAD> {
    pub(crate) fn new_sync(id: u32, ke: &'static keyexpr, timeout: Duration) -> Self {
        Self {
            id,
            ke,
            timeout,
            inner: ZQueryInner::Sync,
        }
    }

    pub(crate) fn new_async(
        id: u32,
        ke: &'static keyexpr,
        timeout: Duration,
        rx: DynamicReceiver<'static, ZOwnedReply<MAX_KEYEXPR, MAX_PAYLOAD>>,
    ) -> Self {
        ZQuery {
            id,
            ke,
            timeout,
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
        match select(
            async {
                embassy_time::Timer::after(self.timeout).await;
                Err(zenoh_proto::ZError::Timeout)
            },
            async {
                match &self.inner {
                    ZQueryInner::Sync => Err(zenoh_proto::ZError::CouldNotRecvFromChannel),
                    ZQueryInner::Async(rx) => Ok(rx.receive().await),
                }
            },
        )
        .await
        {
            embassy_futures::select::Either::First(timeout_err) => timeout_err,
            embassy_futures::select::Either::Second(reply_res) => reply_res,
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
