use core::hint::unreachable_unchecked;

use crate::{config::ZSessionConfig, io::transport::TransportLink};

pub struct SessionResources<'ext, Config>
where
    Config: ZSessionConfig,
{
    pub(crate) inner: SessionResourcesInner<'ext, Config>,
}

impl<Config> Default for SessionResources<'_, Config>
where
    Config: ZSessionConfig,
{
    fn default() -> Self {
        Self {
            inner: SessionResourcesInner::default(),
        }
    }
}

#[derive(Default)]
pub enum SessionResourcesInner<'ext, Config>
where
    Config: ZSessionConfig,
{
    #[default]
    Uninit,
    Init {
        transport: TransportLink<'ext, Config::LinkManager, Config::Buff>,
    },
}

impl<'ext, Config> SessionResources<'ext, Config>
where
    Config: ZSessionConfig,
{
    pub fn init(
        &mut self,
        transport: TransportLink<'ext, Config::LinkManager, Config::Buff>,
    ) -> &mut TransportLink<'ext, Config::LinkManager, Config::Buff> {
        self.inner = SessionResourcesInner::Init { transport };

        match &mut self.inner {
            SessionResourcesInner::Init { transport } => transport,
            _ => unsafe { unreachable_unchecked() },
        }
    }
}
