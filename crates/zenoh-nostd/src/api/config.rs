use crate::platform::ZPlatform;

macro_rules! zdriver {
    ($ty:ty, $ty2:ident) => {
        <$ty as $crate::api::config::ZDriverConfig>::$ty2
    };
}
pub(crate) use zdriver;

pub trait ZDriverConfig {
    type Platform: ZPlatform;

    type TxBuf: AsMut<[u8]>;
    type RxBuf: AsMut<[u8]>;

    fn platform(self) -> Self::Platform;

    fn tx() -> Self::TxBuf;
    fn rx() -> Self::RxBuf;
}

pub trait ZConfig {
    type DriverConfig: ZDriverConfig;
}
