use crate::{config::ZSessionConfig, io::TransportLink};

pub struct SessionResources<'ext, Config>
where
    Config: ZSessionConfig,
{
    pub(crate) inner: SessionResourcesInner<'ext, Config>,
}

impl<'ext, Config> SessionResources<'ext, Config>
where
    Config: ZSessionConfig,
{
    pub fn new() -> Self {
        Self {
            inner: SessionResourcesInner::Uninit,
        }
    }
}

pub enum SessionResourcesInner<'ext, Config>
where
    Config: ZSessionConfig,
{
    Uninit,
    Init {
        transport: TransportLink<'ext, Config::LinkManager, Config::Buff>,
    },
}
