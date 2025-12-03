use zenoh_proto::{
    Encoding, WireExpr,
    exts::Attachment,
    fields::Timestamp,
    keyexpr,
    msgs::{Push, PushBody, Put},
};

use crate::{api::driver::Driver, platform::ZPlatform};

pub struct SessionPutBuilder<'a, 'b, Platform, TxBuf, RxBuf>
where
    Platform: ZPlatform,
{
    driver: &'b Driver<'a, Platform, TxBuf, RxBuf>,
    ke: &'b keyexpr,
    payload: &'b [u8],
    encoding: Encoding<'b>,
    timestamp: Option<Timestamp>,
    attachment: Option<Attachment<'b>>,
}

impl<'a, 'b, Platform, TxBuf, RxBuf> SessionPutBuilder<'a, 'b, Platform, TxBuf, RxBuf>
where
    Platform: ZPlatform,
{
    pub(crate) fn new(
        driver: &'b Driver<'a, Platform, TxBuf, RxBuf>,
        ke: &'b keyexpr,
        payload: &'b [u8],
    ) -> Self {
        Self {
            driver,
            ke,
            payload,
            encoding: Encoding::default(),
            timestamp: None,
            attachment: None,
        }
    }

    pub fn encoding(mut self, encoding: Encoding<'b>) -> Self {
        self.encoding = encoding;
        self
    }

    pub fn timestamp(mut self, timestamp: Timestamp) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    pub fn attachment(mut self, attachment: &'b [u8]) -> Self {
        self.attachment = Some(Attachment { buffer: attachment });
        self
    }
}

impl<'a, 'b, Platform, TxBuf, RxBuf> SessionPutBuilder<'a, 'b, Platform, TxBuf, RxBuf>
where
    Platform: ZPlatform,
    TxBuf: AsMut<[u8]>,
{
    pub async fn finish(self) -> crate::ZResult<()> {
        let msg = Push {
            wire_expr: WireExpr::from(self.ke),
            payload: PushBody::Put(Put {
                payload: self.payload,
                encoding: self.encoding,
                timestamp: self.timestamp,
                attachment: self.attachment,
                ..Default::default()
            }),
            timestamp: self.timestamp,
            ..Default::default()
        };

        self.driver.send(msg).await
    }
}

impl<'a, Platform, TxBuf, RxBuf> super::Session<'a, Platform, TxBuf, RxBuf>
where
    Platform: ZPlatform,
    TxBuf: AsMut<[u8]>,
{
    pub fn put<'b>(
        &'b self,
        ke: &'b keyexpr,
        bytes: &'b [u8],
    ) -> SessionPutBuilder<'a, 'b, Platform, TxBuf, RxBuf> {
        SessionPutBuilder::new(&self.driver, ke, bytes)
    }
}
