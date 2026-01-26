use core::hint::unreachable_unchecked;

use crate::{
    api::{Session, ZConfig, callbacks::*, driver::*},
    io::{TransportLink, ZLinkManager},
};

use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embassy_time::Instant;

#[derive(Default)]
pub enum ResourcesInner<'a, Config, LinkManager>
where
    Config: ZConfig,
    LinkManager: ZLinkManager,
{
    #[default]
    Uninit,
    Init {
        #[allow(unused)]
        config: Config,
        transport: TransportLink<'a, LinkManager>,
    },
}

pub struct Resources<Config>(ResourcesInner<Config>)
where
    Config: ZConfig;

impl<Config> Default for Resources<Config>
where
    Config: ZConfig,
{
    fn default() -> Self {
        Self(ResourcesInner::default())
    }
}

impl<Config> Resources<Config>
where
    Config: ZConfig,
{
    pub(crate) fn init(
        &mut self,
        config: Config,
        transport: TransportLink<Config>,
    ) -> Session<'_, Config> {
        self.0 = ResourcesInner::Init { config, transport };

        let transport_ref_mut = match &mut self.0 {
            ResourcesInner::Init { transport, .. } => transport,
            _ => unsafe { unreachable_unchecked() },
        };

        let (tx, rx) = transport_ref_mut.split();
        let (tx, rx) = (
            DriverTx {
                tx,
                next_keepalive: Instant::now(),
            },
            DriverRx {
                rx,
                last_read: Instant::now(),
            },
        );

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
