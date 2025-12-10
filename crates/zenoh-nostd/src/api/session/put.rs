use zenoh_proto::{exts::*, fields::*, msgs::*, *};

use crate::api::{ZConfig, driver::Driver};

pub struct PutBuilder<'a, 'r, Config>
where
    Config: ZConfig,
{
    driver: &'a Driver<'r, Config>,
    ke: &'a keyexpr,
    payload: &'a [u8],
    encoding: Encoding<'a>,
    timestamp: Option<Timestamp>,
    attachment: Option<Attachment<'a>>,
}

impl<'a, 'r, Config> PutBuilder<'a, 'r, Config>
where
    Config: ZConfig,
{
    pub(crate) fn new(driver: &'a Driver<'r, Config>, ke: &'a keyexpr, payload: &'a [u8]) -> Self {
        Self {
            driver,
            ke,
            payload,
            encoding: Encoding::default(),
            timestamp: None,
            attachment: None,
        }
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

impl<'r, Config> super::Session<'r, Config>
where
    Config: ZConfig,
{
    pub fn put<'a>(&'a self, ke: &'a keyexpr, bytes: &'a [u8]) -> PutBuilder<'a, 'r, Config> {
        PutBuilder::new(self.driver, ke, bytes)
    }
}
