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

    type GetCallbacks<'res>: ZCallbacks<'res, ResponseRef>;
    type SubCallbacks<'res>: ZCallbacks<'res, SampleRef>;
    type QueryableCallbacks<'res>: ZCallbacks<'res, QueryRef<'res, Self>>;

    type TxBuf: AsMut<[u8]>;
    type RxBuf: AsMut<[u8]>;

    fn platform(&self) -> &Self::Platform;

    fn txrx(&mut self) -> (&mut Self::TxBuf, &mut Self::RxBuf);

    fn into_parts(self) -> (Self::Platform, Self::TxBuf, Self::RxBuf);
}
