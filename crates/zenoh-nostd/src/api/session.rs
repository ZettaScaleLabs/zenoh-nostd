use core::hint::unreachable_unchecked;

use zenoh_proto::Endpoint;

use crate::{
    config::ZSessionConfig,
    io::{Driver, TransportLink},
    resources::SessionResourcesInner,
    session::SessionResources,
};

pub struct Session<'ext, 'res, Config>
where
    Config: ZSessionConfig,
{
    pub(crate) driver: Driver<'ext, 'res, Config::LinkManager, Config::Buff>,
}

pub async fn session_connect<'ext, 'res, Config>(
    resources: &'res mut SessionResources<'ext, Config>,
    config: &'ext Config,
    endpoint: Endpoint<'_>,
) -> Session<'ext, 'res, Config>
where
    Config: ZSessionConfig,
{
    let transport: TransportLink<'ext, Config::LinkManager, Config::Buff> = config
        .transports()
        .connect(endpoint, config.buff())
        .await
        .unwrap();

    resources.inner = SessionResourcesInner::Init { transport };
    let transport = match &mut resources.inner {
        SessionResourcesInner::Init { transport } => transport,
        _ => unsafe { unreachable_unchecked() },
    };

    let driver: Driver<'ext, 'res, Config::LinkManager, Config::Buff> = Driver::new(transport);

    Session { driver }
}

pub async fn session_listen<'ext, 'res, Config>(
    resources: &'res mut SessionResources<'ext, Config>,
    config: &'ext Config,
    endpoint: Endpoint<'_>,
) -> Session<'ext, 'res, Config>
where
    Config: ZSessionConfig,
{
    let transport: TransportLink<'ext, Config::LinkManager, Config::Buff> = config
        .transports()
        .listen(endpoint, config.buff())
        .await
        .unwrap();

    resources.inner = SessionResourcesInner::Init { transport };
    let transport = match &mut resources.inner {
        SessionResourcesInner::Init { transport } => transport,
        _ => unsafe { unreachable_unchecked() },
    };

    let driver: Driver<'ext, 'res, Config::LinkManager, Config::Buff> = Driver::new(transport);

    Session { driver }
}
