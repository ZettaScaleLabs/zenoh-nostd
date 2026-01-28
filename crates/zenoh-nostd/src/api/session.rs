// use crate::api::callbacks::ZCallbacks;
use embassy_sync::{
    blocking_mutex::raw::NoopRawMutex,
    mutex::{Mutex, MutexGuard},
};
use zenoh_proto::{Endpoint, TransportLinkError};

use crate::{
    api::callbacks::ZCallbacks,
    // session::SessionResources,
    config::ZSessionConfig,
    io::{driver::Driver, transport::TransportLink},
    platform::ZLinkManager,
    resources::SessionResources,
};

mod run;

pub mod get;
pub mod r#pub;
pub mod put;
pub mod querier;
pub mod queryable;
pub mod sub;

pub(crate) struct SessionState<'res, Config>
where
    Config: ZSessionConfig + 'res,
{
    next: u32,
    sub_callbacks: Config::SubCallbacks<'res>,
    get_callbacks: Config::GetCallbacks<'res>,
    queryable_callbacks: Config::QueryableCallbacks<'res>,
}

impl<'res, Config> SessionState<'res, Config>
where
    Config: ZSessionConfig,
{
    pub fn new() -> Self {
        Self {
            next: 0,
            sub_callbacks: Config::SubCallbacks::empty(),
            get_callbacks: Config::GetCallbacks::empty(),
            queryable_callbacks: Config::QueryableCallbacks::empty(),
        }
    }

    pub(crate) fn next(&mut self) -> u32 {
        let next = self.next;
        self.next += 1;
        next
    }
}

pub struct Session<'res, Config>
where
    Config: ZSessionConfig,
{
    driver: Driver<'res, Config>,
    state: Mutex<NoopRawMutex, SessionState<'res, Config>>,
}

type Link<'res, Config> = <<Config as ZSessionConfig>::LinkManager as ZLinkManager>::Link<'res>;

impl<'res, Config> Session<'res, Config>
where
    Config: ZSessionConfig,
{
    pub fn new(transport: &'res mut TransportLink<Link<'res, Config>, Config::Buff>) -> Self {
        Self {
            driver: Driver::new(transport),
            state: Mutex::new(SessionState::new()),
        }
    }

    pub(crate) async fn state(&self) -> MutexGuard<'_, NoopRawMutex, SessionState<'res, Config>> {
        self.state.lock().await
    }
}

pub async fn session_connect<'res, Config>(
    resources: &'res mut SessionResources<'res, Config>,
    config: &'res Config,
    endpoint: Endpoint<'_>,
) -> core::result::Result<Session<'res, Config>, TransportLinkError>
where
    Config: ZSessionConfig,
{
    Ok(Session::new(resources.init(
        config.transports().connect(endpoint, config.buff()).await?,
    )))
}

pub async fn session_listen<'res, Config>(
    resources: &'res mut SessionResources<'res, Config>,
    config: &'res Config,
    endpoint: Endpoint<'_>,
) -> core::result::Result<Session<'res, Config>, TransportLinkError>
where
    Config: ZSessionConfig,
{
    Ok(Session::new(resources.init(
        config.transports().listen(endpoint, config.buff()).await?,
    )))
}

#[macro_export]
macro_rules! __session_connect {
    (
        $CONFIG:ty: $config:expr,
        $endpoint:expr
    ) => {{
        static CONFIG: static_cell::StaticCell<$CONFIG> = static_cell::StaticCell::new();
        let config = CONFIG.init($config);

        static RESOURCES: static_cell::StaticCell<
            $crate::session::SessionResources<'static, $CONFIG>,
        > = static_cell::StaticCell::new();

        static SESSION: static_cell::StaticCell<$crate::session::zenoh::Session<'static, $CONFIG>> =
            static_cell::StaticCell::new();

        SESSION.init($crate::session::zenoh::Session::new(
            RESOURCES
                .init($crate::session::SessionResources::default())
                .init(
                    config
                        .transports()
                        .connect($endpoint, config.buff())
                        .await?,
                ),
        )) as &$crate::session::zenoh::Session<'static, $CONFIG>
    }};
}

#[macro_export]
macro_rules! __session_listen {
    (
        $CONFIG:ty: $config:expr,
        $endpoint:expr
    ) => {{
        static CONFIG: static_cell::StaticCell<$CONFIG> = static_cell::StaticCell::new();
        let config = CONFIG.init($config);

        static RESOURCES: static_cell::StaticCell<
            $crate::session::SessionResources<'static, $CONFIG>,
        > = static_cell::StaticCell::new();

        static SESSION: static_cell::StaticCell<$crate::session::zenoh::Session<'static, $CONFIG>> =
            static_cell::StaticCell::new();

        SESSION.init($crate::session::zenoh::Session::new(
            RESOURCES
                .init($crate::session::SessionResources::default())
                .init(config.transports().listen($endpoint, config.buff()).await?),
        )) as &$crate::session::zenoh::Session<'static, $CONFIG>
    }};
}

pub async fn session_connect_ignore_invalid_sn<'res, Config>(
    resources: &'res mut SessionResources<'res, Config>,
    config: &'res Config,
    endpoint: Endpoint<'_>,
) -> core::result::Result<Session<'res, Config>, TransportLinkError>
where
    Config: ZSessionConfig,
{
    let mut transport = config.transports().connect(endpoint, config.buff()).await?;
    transport.transport_mut().rx.ignore_invalid_sn();

    Ok(Session::new(resources.init(transport)))
}

pub async fn session_listen_ignore_invalid_sn<'res, Config>(
    resources: &'res mut SessionResources<'res, Config>,
    config: &'res Config,
    endpoint: Endpoint<'_>,
) -> core::result::Result<Session<'res, Config>, TransportLinkError>
where
    Config: ZSessionConfig,
{
    let mut transport = config.transports().listen(endpoint, config.buff()).await?;
    transport.transport_mut().rx.ignore_invalid_sn();
    Ok(Session::new(resources.init(transport)))
}
