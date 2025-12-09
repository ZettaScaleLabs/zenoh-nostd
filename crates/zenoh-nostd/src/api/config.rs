use crate::{
    api::{SamplePtr, ZCallbacks, ZChannels},
    platform::ZPlatform,
};

pub trait ZDriverConfig {
    type Platform: ZPlatform;

    type TxBuf: AsMut<[u8]>;
    type RxBuf: AsMut<[u8]>;

    fn platform(&self) -> &Self::Platform;

    fn txrx(&mut self) -> (&mut Self::TxBuf, &mut Self::RxBuf);
}

pub trait ZSessionConfig {
    type SubscriberCallbacks: ZCallbacks<SamplePtr, ()>;
    type SubscriberChannels: ZChannels<SamplePtr>;
}

pub trait ZConfig: ZDriverConfig + ZSessionConfig {
    fn into_parts(self) -> (Self::Platform, Self::TxBuf, Self::RxBuf);
}
