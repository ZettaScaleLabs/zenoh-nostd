pub(crate) mod run;
pub(crate) mod send;
pub(crate) mod update;

use core::ops::DerefMut;

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embassy_time::Instant;
use zenoh_proto::{ZResult, keyexpr};

use crate::{
    ZQueryableCallback, ZRepliesCallback,
    io::transport::{TransportConfig, TransportRx, TransportTx},
    platform::Platform,
    queryable::ZQueryableCallbacks,
    replies::callback::ZRepliesCallbacks,
    subscriber::callback::{ZSubscriberCallback, ZSubscriberCallbacks},
};

pub struct TxState<T: Platform + 'static> {
    tx_zbuf: &'static mut [u8],
    tx: TransportTx<'static, T>,
    sn: u32,

    next_keepalive: Instant,
}

pub struct RxState<T: Platform + 'static> {
    rx_zbuf: &'static mut [u8],
    rx: TransportRx<'static, T>,
}

pub struct SubscriberState {
    callbacks: &'static mut dyn ZSubscriberCallbacks,
}

pub struct RepliesState {
    callbacks: &'static mut dyn ZRepliesCallbacks,
}

pub struct QueryableState<T: Platform + 'static> {
    callbacks: &'static mut dyn ZQueryableCallbacks<T>,
}

pub struct SessionDriver<T: Platform + 'static> {
    config: TransportConfig,

    tx: Mutex<CriticalSectionRawMutex, TxState<T>>,
    rx: Mutex<CriticalSectionRawMutex, RxState<T>>,

    subscribers: Mutex<CriticalSectionRawMutex, SubscriberState>,
    replies: Mutex<CriticalSectionRawMutex, RepliesState>,
    queryables: Mutex<CriticalSectionRawMutex, QueryableState<T>>,
}

impl<T: Platform> SessionDriver<T> {
    pub(crate) fn new(
        config: TransportConfig,
        tx: (&'static mut [u8], TransportTx<'static, T>),
        rx: (&'static mut [u8], TransportRx<'static, T>),
        subscribers: &'static mut dyn ZSubscriberCallbacks,
        replies: &'static mut dyn ZRepliesCallbacks,
        queryables: &'static mut dyn ZQueryableCallbacks<T>,
    ) -> SessionDriver<T> {
        SessionDriver {
            tx: Mutex::new(TxState {
                tx_zbuf: tx.0,
                tx: tx.1,
                sn: config.negociated_config.mine_sn,
                next_keepalive: Instant::now(),
            }),
            rx: Mutex::new(RxState {
                rx_zbuf: rx.0,
                rx: rx.1,
            }),
            subscribers: Mutex::new(SubscriberState {
                callbacks: subscribers,
            }),
            replies: Mutex::new(RepliesState { callbacks: replies }),
            queryables: Mutex::new(QueryableState {
                callbacks: queryables,
            }),
            config,
        }
    }

    pub(crate) async fn register_subscriber_callback(
        &self,
        id: u32,
        ke: &'static keyexpr,
        callback: ZSubscriberCallback,
    ) -> ZResult<()> {
        let mut cb_guard = self.subscribers.lock().await;
        let cb = cb_guard.deref_mut();

        cb.callbacks.insert(id, ke, callback).map(|_| ())
    }

    pub(crate) async fn register_query_callback(
        &self,
        id: u32,
        ke: &'static keyexpr,
        callback: ZRepliesCallback,
    ) -> ZResult<()> {
        let mut cb_guard = self.replies.lock().await;
        let cb = cb_guard.deref_mut();

        cb.callbacks.drop_timedout();
        cb.callbacks.insert(id, ke, callback).map(|_| ())
    }

    pub async fn remove_query_callback(&self, id: u32) {
        let mut cb_guard = self.replies.lock().await;
        let cb = cb_guard.deref_mut();

        cb.callbacks.remove(&id);
    }

    pub(crate) async fn register_queryable_callback(
        &self,
        id: u32,
        ke: &'static keyexpr,
        callback: ZQueryableCallback<T>,
    ) -> ZResult<()> {
        let mut cb_guard = self.queryables.lock().await;
        let cb = cb_guard.deref_mut();

        cb.callbacks.insert(id, ke, callback).map(|_| ())
    }
}
