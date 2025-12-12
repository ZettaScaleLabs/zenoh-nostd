use crate::{
    api::{EndPoint, ZConfig, driver::Driver},
    io::{
        link::Link,
        transport::{Transport, TransportMineConfig},
    },
};

use embassy_time::Duration;

pub(crate) mod driver;

mod resources;
pub use resources::*;

mod put;
mod run;

mod sub;
pub use sub::{HeaplessSubscriberCallbacks, HeaplessSubscriberChannels, Subscriber};

mod r#pub;
pub use r#pub::Publisher;

mod get;
pub use get::{Get, HeaplessGetCallbacks, HeaplessGetChannels};

mod queryable;
pub use queryable::{HeaplessQueryableCallbacks, HeaplessQueryableChannels, Queryable};

pub struct Session<'a, Config>
where
    Config: ZConfig,
{
    pub(crate) driver: &'a Driver<'a, Config>,
    pub(crate) resources: &'a SessionResources<Config>,
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
