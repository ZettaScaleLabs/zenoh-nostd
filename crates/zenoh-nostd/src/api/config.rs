use crate::{
    api::{
        arg::{QueryRef, ResponseRef, SampleRef},
        callbacks::ZCallbacks,
    },
    io::ZLinkManager,
};

pub trait ZTransportConfig {
    type Buff: AsMut<[u8]> + AsRef<[u8]> + Clone;
    type LinkManager: ZLinkManager;

    fn into_inner(self) -> (Self::Buff, Self::Buff, Self::LinkManager);
}

pub trait ZConfig: Sized {
    type GetCallbacks<'res>: ZCallbacks<'res, ResponseRef>;
    type SubCallbacks<'res>: ZCallbacks<'res, SampleRef>;

    type QueryableCallbacks<'res>: ZCallbacks<'res, QueryRef<'res, Self>>
    where
        Self: 'res;
}
