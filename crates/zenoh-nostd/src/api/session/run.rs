use crate::platform::ZPlatform;

impl<'a, Platform, TxBuf, RxBuf> super::Session<'a, Platform, TxBuf, RxBuf>
where
    Platform: ZPlatform,
    TxBuf: AsMut<[u8]>,
    RxBuf: AsMut<[u8]>,
{
    pub async fn run(&self) -> crate::ZResult<()> {
        self.driver.run().await
    }
}
