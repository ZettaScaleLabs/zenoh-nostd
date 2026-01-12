use crate::{
    api::{EndPoint, Resources, ZConfig, driver::Driver, resources::SessionResources},
    io::{
        link::Link,
        transport::{Transport, TransportMineConfig},
    },
};

use embassy_time::Duration;

mod get;
mod r#pub;
mod put;
mod querier;
mod sub;

pub struct Session<'this, 'res, Config>
where
    Config: ZConfig,
{
    pub(crate) driver: &'this Driver<'this, Config>,
    pub(crate) resources: &'this SessionResources<'res, Config>,
}

impl<Config> Session<'_, '_, Config>
where
    Config: ZConfig,
{
    pub async fn run(&self) -> crate::ZResult<()> {
        self.driver.run(self.resources).await?;

        todo!("implement a `session.close` method that should undeclare all resources")
    }
}

impl<Config> Clone for Session<'_, '_, Config>
where
    Config: ZConfig,
{
    fn clone(&self) -> Self {
        Session {
            driver: self.driver,
            resources: self.resources,
        }
    }
}

/// Create a session bounded to the lifetimes of the `zenoh_nostd::Resources`.
pub async fn open<'this, 'res, Config>(
    resources: &'this mut Resources<'this, 'res, Config>,
    mut config: Config,
    endpoint: EndPoint<'_>,
) -> crate::ZResult<Session<'this, 'res, Config>>
where
    Config: ZConfig,
{
    let link = Link::new(config.platform(), endpoint).await?;

    let (tx, rx) = config.txrx();
    let (transport, tconfig) = Transport::open(
        link,
        TransportMineConfig {
            mine_zid: Default::default(),
            mine_lease: Duration::from_secs(20),
            keep_alive: 4,
            open_timeout: Duration::from_secs(5),
        },
        tx,
        rx,
    )
    .await?;

    Ok(resources.init(config, transport, tconfig))
}

/// Alternative version of `zenoh_nostd::open` that creates an `'static` `zenoh_nostd::Session`.
#[macro_export]
macro_rules! open {
    (
        $config:expr => $CONFIG:ty,
        $endpoint:expr
    ) => {{
        static RESOURCES: static_cell::StaticCell<$crate::Resources<'static, 'static, $CONFIG>> =
            static_cell::StaticCell::new();
        $crate::open(RESOURCES.init($crate::Resources::new()), $config, $endpoint)
    }};
}
