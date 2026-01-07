use crate::{
    api::{EndPoint, ZConfig, session::driver::Driver},
    io::{
        link::Link,
        transport::{Transport, TransportMineConfig},
    },
};

use embassy_time::Duration;

mod driver;
mod resources;

mod put;

pub use resources::*;

pub struct Session<'a, Config>
where
    Config: ZConfig,
{
    pub(crate) driver: &'a Driver<'a, Config>,
    pub(crate) resources: &'a SessionResources<Config>,
}

impl<Config> Session<'_, Config>
where
    Config: ZConfig,
{
    pub async fn run(&self) -> crate::ZResult<()> {
        self.driver.run(self.resources).await?;

        todo!("implement a `session.close` method that should undeclare all resources")
    }
}

impl<'a, Config> Clone for Session<'a, Config>
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

pub async fn open<'a, Config>(
    resources: &'a mut Resources<'a, Config>,
    mut config: Config,
    endpoint: EndPoint<'_>,
) -> crate::ZResult<Session<'a, Config>>
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

    let (driver, resources) = resources.init(config, transport, tconfig);

    Ok(Session { driver, resources })
}
