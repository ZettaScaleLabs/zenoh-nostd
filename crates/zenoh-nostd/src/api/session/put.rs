use zenoh_proto::{exts::*, fields::*, msgs::*, *};

use crate::api::{ZConfig, driver::Driver};

pub struct PutBuilder<'a, 'res, Config>
where
    Config: ZConfig,
{
    pub(crate) driver: &'a Driver<'res, Config>,

    pub(crate) ke: &'a keyexpr,
    pub(crate) payload: &'a [u8],

    pub(crate) encoding: Encoding<'a>,
    pub(crate) timestamp: Option<Timestamp>,
    pub(crate) attachment: Option<Attachment<'a>>,
}

impl<'a, 'res, Config> PutBuilder<'a, 'res, Config>
where
    Config: ZConfig,
{
    pub(crate) fn new(
        driver: &'a Driver<'res, Config>,
        ke: &'a keyexpr,
        payload: &'a [u8],
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

    pub fn payload(mut self, payload: &'a [u8]) -> Self {
        self.payload = payload;
        self
    }

    pub fn encoding(mut self, encoding: Encoding<'a>) -> Self {
        self.encoding = encoding;
        self
    }

    pub fn timestamp(mut self, timestamp: Timestamp) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    pub fn attachment(mut self, attachment: &'a [u8]) -> Self {
        self.attachment = Some(Attachment { buffer: attachment });
        self
    }

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

impl<'res, Config> super::Session<'res, Config>
where
    Config: ZConfig,
{
    pub fn put<'a>(&'a self, ke: &'a keyexpr, payload: &'a [u8]) -> PutBuilder<'a, 'res, Config> {
        PutBuilder::new(&self.driver, ke, payload)
    }
}
