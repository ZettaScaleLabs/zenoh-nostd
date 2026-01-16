use crate::{
    api::{
        arg::{QueryRef, ResponseRef, SampleRef},
        callbacks::ZCallbacks,
    },
    platform::ZPlatform,
};

pub trait ZConfig
where
    Self: Sized + 'static,
{
    type Platform: ZPlatform;
    type Buff: AsMut<[u8]> + AsRef<[u8]> + Clone;

    type GetCallbacks<'res>: ZCallbacks<'res, ResponseRef>;
    type SubCallbacks<'res>: ZCallbacks<'res, SampleRef>;
    type QueryableCallbacks<'res>: ZCallbacks<'res, QueryRef<'res, Self>>;

    fn platform(&self) -> &Self::Platform;
    fn buff(&self) -> Self::Buff;
    fn into_platform(self) -> Self::Platform;
}
