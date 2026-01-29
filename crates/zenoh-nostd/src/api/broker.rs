use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use heapless::FnvIndexMap;
use zenoh_proto::{Endpoint, fields::ZenohIdProto};

use crate::{
    config::{ZBrokerConfig, ZSessionConfig},
    io::driver::Driver,
    platform::ZLinkManager,
};

type Link<Config> = <<Config as ZBrokerConfig>::LinkManager as ZLinkManager>::Link<'static>;
type StaticDriver<Config> =
    crate::io::driver::Driver<'static, Link<Config>, <Config as ZBrokerConfig>::Buff>;

pub struct BrokerState<Config>
where
    Config: ZBrokerConfig + 'static,
{
    south: FnvIndexMap<ZenohIdProto, &'static StaticDriver<Config>, 10>,
}

pub struct Broker<Config>
where
    Config: ZBrokerConfig + 'static,
{
    config: &'static Config,
    state: Mutex<NoopRawMutex, BrokerState<Config>>,
}

impl<Config> Broker<Config>
where
    Config: ZBrokerConfig,
{
    pub fn new(config: &'static Config) -> Self {
        Self {
            config,
            state: Mutex::new(BrokerState {
                south: FnvIndexMap::new(),
            }),
        }
    }

    pub async fn run(&self) {}

    pub async fn open(&self, endpoint: Endpoint<'_>) {
        let mut transport = self
            .config
            .transports()
            .listen(endpoint, self.config.buff())
            .await
            .unwrap();

        let driver = Driver::new(&mut transport);

        driver
            .run(&self.state, async |_, _, _, _| Ok::<(), usize>(()))
            .await
            .unwrap();
    }
}

#[macro_export]
macro_rules! __broker {
    ($CONFIG:ty: $config:expr) => {
        ()
    };
}

#[macro_export]
macro_rules! __broker_connect {
    ($broker:expr, $endpoint:expr) => {
        ()
    };
}

#[macro_export]
macro_rules! __broker_listen {
    ($endpoint:expr, $broker:expr) => {
        ()
    };
}
