use alloc::collections::BTreeMap;
use alloc::{boxed::Box, sync::Arc};

use embassy_sync::{
    blocking_mutex::raw::NoopRawMutex,
    mutex::{Mutex, MutexGuard},
};
use zenoh_proto::BrokerError;
use zenoh_proto::msgs::NetworkMessage;
use zenoh_proto::{Endpoint, fields::ZenohIdProto};

use crate::{config::ZBrokerConfig, io::driver::Driver, platform::ZLinkManager};

type Link<Config> = <<Config as ZBrokerConfig>::LinkManager as ZLinkManager>::Link<'static>;
type StaticDriver<Config> = Driver<'static, Link<Config>, <Config as ZBrokerConfig>::Buff>;

pub struct BrokerState<Config>
where
    Config: ZBrokerConfig + 'static,
{
    north: Option<(ZenohIdProto, Arc<StaticDriver<Config>>)>,
    south: BTreeMap<ZenohIdProto, Arc<StaticDriver<Config>>>,
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
                north: None,
                south: BTreeMap::new(),
            }),
        }
    }

    pub(crate) async fn state(&self) -> MutexGuard<'_, NoopRawMutex, BrokerState<Config>> {
        self.state.lock().await
    }

    async fn update(
        north: bool,
        id: ZenohIdProto,
        state: &mut BrokerState<Config>,
        msg: NetworkMessage<'_>,
        bytes: &[u8],
    ) -> core::result::Result<(), BrokerError> {
        let _ = (north, id, state, msg, bytes);
        Ok(())
    }

    async fn update_north(
        id: ZenohIdProto,
        state: &mut BrokerState<Config>,
        msg: NetworkMessage<'_>,
        bytes: &[u8],
    ) -> core::result::Result<(), BrokerError> {
        Self::update(true, id, state, msg, bytes).await
    }

    async fn update_south(
        id: ZenohIdProto,
        state: &mut BrokerState<Config>,
        msg: NetworkMessage<'_>,
        bytes: &[u8],
    ) -> core::result::Result<(), BrokerError> {
        Self::update(false, id, state, msg, bytes).await
    }

    pub async fn connect(&self, endpoint: Endpoint<'_>) -> core::result::Result<(), BrokerError> {
        let driver = Arc::new(Driver::new(Box::leak(Box::new(
            self.config
                .transports()
                .connect(endpoint.clone(), self.config.buff())
                .await?,
        ))));

        self.state().await.north = Some((driver.zid(), driver.clone()));

        let res = driver
            .run(&self.state, Self::update_north)
            .await
            .map_err(|e| e.flatten_map::<BrokerError>());

        self.state().await.north.take();

        res
    }

    pub async fn listen(&self, endpoint: Endpoint<'_>) -> core::result::Result<(), BrokerError> {
        loop {
            let driver = Arc::new(Driver::new(Box::leak(Box::new(
                self.config
                    .transports()
                    .listen(endpoint.clone(), self.config.buff())
                    .await?,
            ))));

            self.state().await.north = Some((driver.zid(), driver.clone()));

            if let Err(e) = driver
                .run(&self.state, Self::update_north)
                .await
                .map_err(|e| e.flatten_map::<BrokerError>())
            {
                zenoh_proto::error!("Error on north: {}", e);
            }

            self.state().await.north.take();
        }
    }

    pub async fn open(&self, endpoint: Endpoint<'_>) -> core::result::Result<(), BrokerError> {
        loop {
            let driver = Arc::new(Driver::new(Box::leak(Box::new(
                self.config
                    .transports()
                    .listen(endpoint.clone(), self.config.buff())
                    .await?,
            ))));

            self.state()
                .await
                .south
                .insert(driver.zid(), driver.clone());

            if let Err(e) = driver
                .run(&self.state, Self::update_south)
                .await
                .map_err(|e| e.flatten_map::<BrokerError>())
            {
                zenoh_proto::error!("Error on south: {}", e);
            }

            self.state().await.south.remove(&driver.zid());
        }
    }
}

#[macro_export]
macro_rules! __broker {
    ($CONFIG:ty: $config:expr) => {{
        static CONFIG: static_cell::StaticCell<$CONFIG> = static_cell::StaticCell::new();
        let config = CONFIG.init($config);

        static BROKER: static_cell::StaticCell<$crate::broker::Broker<$CONFIG>> =
            static_cell::StaticCell::new();

        BROKER.init($crate::broker::Broker::new(config)) as &'static $crate::broker::Broker<$CONFIG>
    }};
}
