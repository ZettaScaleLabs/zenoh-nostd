use crate::{
    EndPoint,
    api::{Resources, ZConfig, driver::Driver, resources::SessionResources},
    io::{
        link::Link,
        transport::{TransportConfig, TransportLink},
    },
};

use embassy_time::Duration;
use zenoh_proto::fields::{Resolution, ZenohIdProto};

mod get;
mod r#pub;
mod put;
mod querier;
mod queryable;
mod sub;

pub struct Session<'res, Config>
where
    Config: ZConfig,
{
    pub(crate) driver: Driver<'res, Config>,
    pub(crate) resources: SessionResources<'res, Config>,
}

impl<Config> Session<'_, Config>
where
    Config: ZConfig,
{
    pub async fn run(&self) -> crate::ZResult<()> {
        self.driver.run(&self.resources).await?;

        todo!("implement a `session.close` method that should undeclare all resources")
    }
}

/// Create a session bounded to the lifetimes of the `zenoh_nocore::Resources`.
pub async fn connect<'res, Config>(
    resources: &'res mut Resources<Config>,
    config: Config,
    endpoint: EndPoint<'_>,
) -> crate::ZResult<Session<'res, Config>>
where
    Config: ZConfig,
{
    let link = Link::connect(config.platform(), endpoint).await?;

    let transport = TransportLink::connect(
        link,
        TransportConfig {
            zid: ZenohIdProto::default(),
            lease: Duration::from_secs(20),
            resolution: Resolution::default(),

            open_timeout: Duration::from_secs(5),
        },
        config.buff(),
    )
    .await?;

    Ok(resources.init(config, transport))
}

/// Create a session bounded to the lifetimes of the `zenoh_nocore::Resources`.
pub async fn listen<'res, Config>(
    resources: &'res mut Resources<Config>,
    config: Config,
    endpoint: EndPoint<'_>,
) -> crate::ZResult<Session<'res, Config>>
where
    Config: ZConfig,
{
    let link = Link::listen(config.platform(), endpoint).await?;

    let transport = TransportLink::listen(
        link,
        TransportConfig {
            zid: ZenohIdProto::default(),
            lease: Duration::from_secs(20),
            resolution: Resolution::default(),

            open_timeout: Duration::from_secs(5),
        },
        config.buff(),
    )
    .await?;

    Ok(resources.init(config, transport))
}

/// Alternative version of `zenoh_nocore::connect` that creates an `'static` `zenoh_nocore::Session`.
#[macro_export]
macro_rules! connect {
    (
        $config:expr => $CONFIG:ty,
        $endpoint:expr
    ) => {{
        static RESOURCES: static_cell::StaticCell<$crate::Resources<$CONFIG>> =
            static_cell::StaticCell::new();

        static SESSION: static_cell::StaticCell<$crate::Session<'static, $CONFIG>> =
            static_cell::StaticCell::new();

        SESSION.init(
            $crate::connect(
                RESOURCES.init($crate::Resources::default()),
                $config,
                $endpoint,
            )
            .await?,
        ) as &'static $crate::Session<'static, $CONFIG>
    }};
}

/// Alternative version of `zenoh_nocore::listen` that creates an `'static` `zenoh_nocore::Session`.
#[macro_export]
macro_rules! listen {
    (
        $config:expr => $CONFIG:ty,
        $endpoint:expr
    ) => {{
        static RESOURCES: static_cell::StaticCell<$crate::Resources<$CONFIG>> =
            static_cell::StaticCell::new();

        static SESSION: static_cell::StaticCell<$crate::Session<'static, $CONFIG>> =
            static_cell::StaticCell::new();

        SESSION.init(
            $crate::listen(
                RESOURCES.init($crate::Resources::default()),
                $config,
                $endpoint,
            )
            .await?,
        ) as &'static $crate::Session<'static, $CONFIG>
    }};
}
