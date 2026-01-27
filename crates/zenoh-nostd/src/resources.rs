use core::hint::unreachable_unchecked;

use crate::{config::ZSessionConfig, io::transport::TransportLink, platform::ZLinkManager};

pub struct SessionResources<'res, Config>
where
    Config: ZSessionConfig,
{
    pub(crate) inner: SessionResourcesInner<'res, Config>,
}

impl<'res, Config> Default for SessionResources<'res, Config>
where
    Config: ZSessionConfig,
{
    fn default() -> Self {
        Self {
            inner: SessionResourcesInner::default(),
        }
    }
}

type Link<'res, Config> = <<Config as ZSessionConfig>::LinkManager as ZLinkManager>::Link<'res>;

#[derive(Default)]
pub enum SessionResourcesInner<'res, Config>
where
    Config: ZSessionConfig + 'res,
{
    #[default]
    Uninit,
    Init {
        transport: TransportLink<Link<'res, Config>, Config::Buff>,
    },
}

impl<'res, Config> SessionResources<'res, Config>
where
    Config: ZSessionConfig,
{
    pub fn init(
        &mut self,
        transport: TransportLink<Link<'res, Config>, Config::Buff>,
    ) -> &mut TransportLink<Link<'res, Config>, Config::Buff> {
        self.inner = SessionResourcesInner::Init { transport };

        match &mut self.inner {
            SessionResourcesInner::Init { transport } => transport,
            _ => unsafe { unreachable_unchecked() },
        }
    }
}
