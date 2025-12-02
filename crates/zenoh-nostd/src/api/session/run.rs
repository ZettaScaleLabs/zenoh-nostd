use crate::{
    api::driver::{Driver, DriverRx, DriverTx},
    io::transport::{TransportRx, TransportTx},
    platform::ZPlatform,
};

impl<'a, Platform, TxBuf, RxBuf>
    super::Session<
        'a,
        Driver<
            DriverTx<TxBuf, TransportTx<'a, Platform>>,
            DriverRx<RxBuf, TransportRx<'a, Platform>>,
        >,
    >
where
    Platform: ZPlatform,
    TxBuf: AsMut<[u8]>,
    RxBuf: AsMut<[u8]>,
{
    pub async fn run(&self) -> crate::ZResult<()> {
        self.driver.run().await
    }
}
