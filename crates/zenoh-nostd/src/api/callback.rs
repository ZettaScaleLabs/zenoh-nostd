use embassy_sync::{
    blocking_mutex::raw::NoopRawMutex,
    channel::{Channel, DynamicReceiver},
};

use crate::api::{OwnedSample, Sample};

pub enum Callback<
    const MAX_KEYEXPR_LEN: usize,
    const MAX_PARAMETERS_LEN: usize,
    const MAX_PAYLOAD_LEN: usize,
    const MAX_QUEUED: usize,
> {
    SyncSubscriber(fn(&Sample)),
    AsyncSubscriber(
        Channel<NoopRawMutex, OwnedSample<MAX_KEYEXPR_LEN, MAX_PAYLOAD_LEN>, MAX_QUEUED>,
    ),
}

impl<const K: usize, const P: usize, const L: usize, const Q: usize> Callback<K, P, L, Q> {
    pub(crate) fn new_sync_subscriber(cb: fn(&Sample)) -> Self {
        Self::SyncSubscriber(cb)
    }

    pub(crate) fn new_async_subscriber() -> Self {
        Self::AsyncSubscriber(Channel::new())
    }

    pub(crate) fn subscriber_receiver(&self) -> Option<DynamicReceiver<'_, OwnedSample<K, L>>> {
        match self {
            Callback::SyncSubscriber(_) => None,
            Callback::AsyncSubscriber(channel) => Some(channel.dyn_receiver()),
        }
    }

    pub(crate) async fn call_subscriber(&self, msg: Sample<'_>) -> crate::ZResult<()> {
        match self {
            Callback::SyncSubscriber(cb) => {
                cb(&msg);
                Ok(())
            }
            Callback::AsyncSubscriber(sender) => {
                let owned_msg = OwnedSample::try_from(msg)?;
                sender.send(owned_msg).await;
                Ok(())
            }
        }
    }
}
