use embassy_sync::{blocking_mutex::raw::NoopRawMutex, channel::DynamicReceiver};
use embassy_time::Instant;
use heapless::FnvIndexMap;
use zenoh_proto::keyexpr;

use crate::{
    api::{
        Callback, OwnedSample, PublicConfig, Sample,
        driver::{Driver, DriverRx, DriverTx},
    },
    io::transport::{Transport, TransportConfig},
    platform::ZPlatform,
};

pub type CallbackId = u32;

type Mutex<T> = embassy_sync::mutex::Mutex<NoopRawMutex, T>;

pub struct SessionResources<
    const MAX_KEYEXPR_LEN: usize,
    const MAX_PARAMETERS_LEN: usize,
    const MAX_PAYLOAD_LEN: usize,
    const MAX_QUEUED: usize,
    const MAX_CALLBACKS: usize,
    const MAX_SUBSCRIBERS: usize,
> {
    next: Mutex<u32>,

    pub callbacks: FnvIndexMap<
        CallbackId,
        Callback<MAX_KEYEXPR_LEN, MAX_PARAMETERS_LEN, MAX_PAYLOAD_LEN, MAX_QUEUED>,
        MAX_CALLBACKS,
    >,

    pub subscribers: Mutex<FnvIndexMap<&'static keyexpr, CallbackId, MAX_SUBSCRIBERS>>,
}

impl<
    const MAX_KEYEXPR_LEN: usize,
    const MAX_PARAMETERS_LEN: usize,
    const MAX_PAYLOAD_LEN: usize,
    const MAX_QUEUED: usize,
    const MAX_CALLBACKS: usize,
    const MAX_SUBSCRIBERS: usize,
>
    SessionResources<
        MAX_KEYEXPR_LEN,
        MAX_PARAMETERS_LEN,
        MAX_PAYLOAD_LEN,
        MAX_QUEUED,
        MAX_CALLBACKS,
        MAX_SUBSCRIBERS,
    >
{
    pub fn new() -> Self {
        Self {
            next: Mutex::new(0),
            callbacks: FnvIndexMap::new(),
            subscribers: Mutex::new(FnvIndexMap::new()),
        }
    }

    pub async fn next_id(&self) -> CallbackId {
        let mut next = self.next.lock().await;
        let id = *next;
        *next = next.wrapping_add(1);
        id
    }

    pub async fn subscriber_sync(&mut self, v: fn(&Sample)) -> crate::ZResult<CallbackId> {
        let id = self.next_id().await;
        self.callbacks
            .insert(id, Callback::new_sync_subscriber(v))
            .map_err(|_| crate::ZError::CapacityExceeded)?;
        Ok(id)
    }

    pub async fn subscriber(&mut self) -> crate::ZResult<CallbackId> {
        let id = self.next_id().await;
        self.callbacks
            .insert(id, Callback::new_async_subscriber())
            .map_err(|_| crate::ZError::CapacityExceeded)?;
        Ok(id)
    }

    pub async fn register_subscriber(
        &self,
        ke: &'static keyexpr,
        cb: CallbackId,
    ) -> crate::ZResult<u32> {
        let id = self.next_id().await;
        let mut self_subscribers = self.subscribers.lock().await;
        self_subscribers
            .insert(ke, cb)
            .map_err(|_| crate::ZError::CapacityExceeded)?;
        Ok(id)
    }

    pub fn subscriber_receiver<'a>(
        &'a self,
        cb: CallbackId,
    ) -> Option<DynamicReceiver<'a, OwnedSample<MAX_KEYEXPR_LEN, MAX_PAYLOAD_LEN>>> {
        let callback = self.callbacks.get(&cb)?;
        callback.subscriber_receiver()
    }
}

pub struct PublicResources<'a, Platform, TxBuf, RxBuf, SessionResources>
where
    Platform: ZPlatform,
{
    transport: Option<Transport<Platform>>,
    driver: Option<Driver<'a, Platform, TxBuf, RxBuf>>,
    session_resources: SessionResources,
}

impl<
    'a,
    Platform,
    TxBuf,
    RxBuf,
    const MAX_KEYEXPR_LEN: usize,
    const MAX_PARAMETERS_LEN: usize,
    const MAX_PAYLOAD_LEN: usize,
    const MAX_QUEUED: usize,
    const MAX_CALLBACKS: usize,
    const MAX_SUBSCRIBERS: usize,
>
    PublicResources<
        'a,
        Platform,
        TxBuf,
        RxBuf,
        SessionResources<
            MAX_KEYEXPR_LEN,
            MAX_PARAMETERS_LEN,
            MAX_PAYLOAD_LEN,
            MAX_QUEUED,
            MAX_CALLBACKS,
            MAX_SUBSCRIBERS,
        >,
    >
where
    Platform: ZPlatform,
{
    pub fn new() -> Self {
        Self {
            transport: None,
            driver: None,
            session_resources: SessionResources::new(),
        }
    }

    pub async fn subscriber_sync(&mut self, v: fn(&Sample)) -> crate::ZResult<CallbackId> {
        self.session_resources.subscriber_sync(v).await
    }

    pub async fn subscriber_async(&mut self) -> crate::ZResult<CallbackId> {
        self.session_resources.subscriber().await
    }
}

impl<'a, Platform, TxBuf, RxBuf, SessionResources>
    PublicResources<'a, Platform, TxBuf, RxBuf, SessionResources>
where
    Platform: ZPlatform,
{
    pub(crate) fn init(
        &'a mut self,
        config: PublicConfig<Platform, TxBuf, RxBuf>,
        transport: Transport<Platform>,
        tconfig: TransportConfig,
    ) -> (&'a Driver<'a, Platform, TxBuf, RxBuf>, &'a SessionResources) {
        let Self {
            transport: t,
            driver: d,
            session_resources: sr,
        } = self;

        *t = Some(transport);
        let (tx, rx) = t.as_mut().expect("Transport just set").split();
        let (tx, rx) = (
            DriverTx {
                tx_buf: config.tx,
                tx,
                sn: tconfig.negociated_config.mine_sn,
                next_keepalive: Instant::now(),
                config: tconfig.mine_config.clone(),
            },
            DriverRx {
                rx_buf: config.rx,
                rx,
                last_read: Instant::now(),
                config: tconfig.other_config.clone(),
            },
        );

        let driver = Driver::new(tx, rx);
        *d = Some(driver);
        (d.as_ref().expect("Driver just set"), sr)
    }
}
