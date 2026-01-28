use crate::{
    api::{
        arg::{GetResponseRef, QueryableQueryRef, SampleRef},
        callbacks::ZCallbacks,
    },
    io::{link::ZLinkManager, transport::TransportLinkManager},
};

pub trait ZConfig {
    type Buff: AsMut<[u8]> + AsRef<[u8]> + Clone;
    type LinkManager: ZLinkManager;

    fn transports(&self) -> &TransportLinkManager<Self::LinkManager>;
    fn buff(&self) -> Self::Buff;
}

pub trait ZSessionConfig: Sized {
    type Buff: AsMut<[u8]> + AsRef<[u8]> + Clone;
    type LinkManager: ZLinkManager;

    type SubCallbacks<'res>: ZCallbacks<'res, SampleRef>;
    type GetCallbacks<'res>: ZCallbacks<'res, GetResponseRef>;
    type QueryableCallbacks<'res>: ZCallbacks<'res, QueryableQueryRef<'res, Self>>
    where
        Self: 'res;

    fn transports(&self) -> &TransportLinkManager<Self::LinkManager>;
    fn buff(&self) -> Self::Buff;
}

impl<T> ZConfig for T
where
    T: ZSessionConfig,
{
    type Buff = <Self as ZSessionConfig>::Buff;
    type LinkManager = <Self as ZSessionConfig>::LinkManager;

    fn transports(&self) -> &TransportLinkManager<Self::LinkManager> {
        ZSessionConfig::transports(self)
    }

    fn buff(&self) -> Self::Buff {
        ZSessionConfig::buff(self)
    }
}
