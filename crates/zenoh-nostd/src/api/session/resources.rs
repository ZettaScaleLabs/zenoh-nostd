use crate::{
    api::{
        ZCallbacks, ZConfig, ZSessionConfig,
        driver::{Driver, DriverRx, DriverTx},
    },
    io::transport::{Transport, TransportConfig},
};

use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embassy_time::Instant;

pub struct SessionResources<Config>
where
    Config: ZSessionConfig,
{
    pub next: Mutex<NoopRawMutex, u32>,

    pub subscribers: Mutex<NoopRawMutex, Config::SubscriberCallbacks>,
}

impl<Config> SessionResources<Config>
where
    Config: ZSessionConfig,
{
    pub async fn next(&self) -> u32 {
        let mut guard = self.next.lock().await;
        let next = *guard;
        *guard += 1;
        next
    }
}

pub struct Resources<'r, Config>
where
    Config: ZConfig,
{
    transport: Option<Transport<Config::Platform>>,
    driver: Option<Driver<'r, Config>>,

    session: SessionResources<Config>,
}

impl<Config> Resources<'_, Config>
where
    Config: ZConfig,
{
    pub fn new() -> Self {
        Self {
            transport: None,
            driver: None,
            session: SessionResources {
                next: Mutex::new(0),
                subscribers: Mutex::new(Config::SubscriberCallbacks::empty()),
            },
        }
    }
}

impl<'r, Config> Resources<'r, Config>
where
    Config: ZConfig,
{
    pub(crate) fn init(
        &'r mut self,
        config: Config,
        transport: Transport<Config::Platform>,
        tconfig: TransportConfig,
    ) -> (&'r Driver<'r, Config>, &'r SessionResources<Config>) {
        let Self {
            transport: t,
            driver: d,
            session: s,
            ..
        } = self;

        let (_, tx_buf, rx_buf) = config.into_parts();

        *t = Some(transport);
        let (tx, rx) = t.as_mut().expect("Transport just set").split();
        let (tx, rx) = (
            DriverTx {
                tx_buf,
                tx,
                sn: tconfig.negociated_config.mine_sn,
                next_keepalive: Instant::now(),
                config: tconfig.mine_config.clone(),
            },
            DriverRx {
                rx_buf,
                rx,
                last_read: Instant::now(),
                config: tconfig.other_config.clone(),
            },
        );

        let driver = Driver::new(tx, rx);
        *d = Some(driver);
        (d.as_ref().expect("Driver just set"), s)
    }
}
