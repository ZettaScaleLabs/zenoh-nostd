use embassy_sync::{blocking_mutex::raw::NoopRawMutex, channel::Channel, mutex::Mutex};
use embassy_time::Instant;
use heapless::FnvIndexMap;
use zenoh_proto::keyexpr;

use crate::{
    api::{
        Callback, PublicConfig, ZOwnedSample, ZSample,
        driver::{Driver, DriverRx, DriverTx},
    },
    io::transport::{Transport, TransportConfig},
    platform::ZPlatform,
};

pub struct SessionResources<
    const MAX_KEYEXPR: usize,
    const MAX_QUEUED: usize,
    const MAX_KEYEXPR_LEN: usize,
    const MAX_PARAMETERS_LEN: usize,
    const MAX_PAYLOAD_LEN: usize,
    const MAX_SUBSCRIPTIONS: usize,
    const MAX_QUERIES: usize,
    const MAX_QUERYABLES: usize,
> {
    keyexpr_lt: FnvIndexMap<&'static keyexpr, u32, MAX_KEYEXPR>,

    next: u32,

    subscriptions: FnvIndexMap<
        u32,
        Callback<fn(&ZSample), ZOwnedSample<MAX_KEYEXPR_LEN, MAX_PAYLOAD_LEN>, MAX_QUEUED>,
        MAX_SUBSCRIPTIONS,
    >,
}

impl<
    const MAX_KEYEXPR: usize,
    const MAX_QUEUED: usize,
    const MAX_KEYEXPR_LEN: usize,
    const MAX_PARAMETERS_LEN: usize,
    const MAX_PAYLOAD_LEN: usize,
    const MAX_SUBSCRIPTIONS: usize,
    const MAX_QUERIES: usize,
    const MAX_QUERYABLES: usize,
>
    SessionResources<
        MAX_KEYEXPR,
        MAX_QUEUED,
        MAX_KEYEXPR_LEN,
        MAX_PARAMETERS_LEN,
        MAX_PAYLOAD_LEN,
        MAX_SUBSCRIPTIONS,
        MAX_QUERIES,
        MAX_QUERYABLES,
    >
{
    pub fn new() -> Self {
        Self {
            keyexpr_lt: FnvIndexMap::new(),
            next: 0,
            subscriptions: FnvIndexMap::new(),
        }
    }

    pub fn next_id(&mut self) -> u32 {
        let id = self.next;
        self.next = self.next.wrapping_add(1);
        id
    }

    pub fn register_sync_subscriber(&mut self, callback: fn(&ZSample)) -> crate::ZResult<u32> {
        let id = self.next_id();
        self.subscriptions
            .insert(id, Callback::new_sync(callback))
            .map_err(|_| crate::ZError::CapacityExceeded)?;
        Ok(id)
    }

    pub fn register_async_subscriber(&mut self) -> crate::ZResult<u32> {
        let id = self.next_id();
        self.subscriptions
            .insert(
                id,
                Callback::new_async(Channel::<
                    NoopRawMutex,
                    ZOwnedSample<MAX_KEYEXPR_LEN, MAX_PAYLOAD_LEN>,
                    MAX_QUEUED,
                >::new()),
            )
            .map_err(|_| crate::ZError::CapacityExceeded)?;
        Ok(id)
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
    const MAX_KEYEXPR: usize,
    const MAX_QUEUED: usize,
    const MAX_KEYEXPR_LEN: usize,
    const MAX_PARAMETERS_LEN: usize,
    const MAX_PAYLOAD_LEN: usize,
    const MAX_SUBSCRIPTIONS: usize,
    const MAX_QUERIES: usize,
    const MAX_QUERYABLES: usize,
>
    PublicResources<
        'a,
        Platform,
        TxBuf,
        RxBuf,
        SessionResources<
            MAX_KEYEXPR,
            MAX_QUEUED,
            MAX_KEYEXPR_LEN,
            MAX_PARAMETERS_LEN,
            MAX_PAYLOAD_LEN,
            MAX_SUBSCRIPTIONS,
            MAX_QUERIES,
            MAX_QUERYABLES,
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
    ) -> &'a Driver<'a, Platform, TxBuf, RxBuf> {
        let Self {
            transport: t,
            driver: d,
            ..
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
        d.as_ref().expect("Driver just set")
    }
}
