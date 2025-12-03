use crate::{api::SessionResources, platform::ZPlatform};

impl<
    'a,
    Platform,
    TxBuf,
    RxBuf,
    const MAX_KEYEXPR_LEN: usize,
    const MAX_PARAMETERS_LEN: usize,
    const MAX_PAYLOAD_LEN: usize,
    const MAX_QUEUED: usize,
    const MAX_CALLBACKS: usize,
    const MAX_SUBSCRIBERS: usize,
>
    super::PublicSession<
        'a,
        Platform,
        TxBuf,
        RxBuf,
        SessionResources<
            MAX_KEYEXPR_LEN,
            MAX_PARAMETERS_LEN,
            MAX_PAYLOAD_LEN,
            MAX_QUEUED,
            MAX_CALLBACKS,
            MAX_SUBSCRIBERS,
        >,
    >
where
    Platform: ZPlatform,
    TxBuf: AsMut<[u8]>,
    RxBuf: AsMut<[u8]>,
{
    pub async fn run(&self) -> crate::ZResult<()> {
        self.driver.run(self.resources).await
    }
}
