use std::hint::unreachable_unchecked;

use crate::{
    api::{Session, ZConfig, callbacks::*, driver::*},
    io::transport::{Transport, TransportConfig},
};

use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embassy_time::Instant;

pub enum ResourcesInner<Config>
where
    Config: ZConfig,
{
    Uninit,
    Init {
        #[allow(unused)]
        platform: Config::Platform,
        transport: Transport<Config::Platform>,
    },
}

pub struct Resources<Config>(ResourcesInner<Config>)
where
    Config: ZConfig;

impl<Config> Resources<Config>
where
    Config: ZConfig,
{
    pub fn new() -> Self {
        Self(ResourcesInner::Uninit)
    }

    pub(crate) fn init(
        &mut self,
        config: Config,
        transport: Transport<Config::Platform>,
        tconfig: TransportConfig,
    ) -> Session<'_, Config> {
        let (platform, tx_buf, rx_buf) = config.into_parts();

        self.0 = ResourcesInner::Init {
            platform,
            transport,
        };

        let transport_ref_mut = match &mut self.0 {
            ResourcesInner::Init { transport, .. } => transport,
            _ => unsafe { unreachable_unchecked() },
        };

        let (tx, rx) = {
            let (tx, rx) = transport_ref_mut.split();
            (
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
            )
        };

        Session {
            driver: Driver::new(tx, rx),
            resources: SessionResources::new(),
        }
    }
}

pub struct SessionResources<'res, Config>
where
    Config: ZConfig,
{
    pub next: Mutex<NoopRawMutex, u32>,
    pub get_callbacks: Mutex<NoopRawMutex, Config::GetCallbacks<'res>>,
    pub sub_callbacks: Mutex<NoopRawMutex, Config::SubCallbacks<'res>>,
    pub queryable_callbacks: Mutex<NoopRawMutex, Config::QueryableCallbacks<'res>>,
}

impl<Config> SessionResources<'_, Config>
where
    Config: ZConfig,
{
    pub fn new() -> Self {
        Self {
            next: Mutex::new(0),
            get_callbacks: Mutex::new(Config::GetCallbacks::empty()),
            sub_callbacks: Mutex::new(Config::SubCallbacks::empty()),
            queryable_callbacks: Mutex::new(Config::QueryableCallbacks::empty()),
        }
    }

    pub async fn next(&self) -> u32 {
        let mut guard = self.next.lock().await;
        let next = *guard;
        *guard += 1;
        next
    }
}
