use crate::{
    api::{SamplePtr, ZCallbacks, ZChannels},
    platform::ZPlatform,
};

pub trait ZConfig {
    type Platform: ZPlatform;
    type SubscriberCallbacks: ZCallbacks<SamplePtr, ()>;
    type SubscriberChannels: ZChannels<SamplePtr>;

    type TxBuf: AsMut<[u8]>;
    type RxBuf: AsMut<[u8]>;

    fn platform(&self) -> &Self::Platform;

    fn txrx(&mut self) -> (&mut Self::TxBuf, &mut Self::RxBuf);

    fn into_parts(self) -> (Self::Platform, Self::TxBuf, Self::RxBuf);
}
