use zenoh_proto::{exts::*, fields::*, msgs::*, *};

use crate::api::{ZConfig, driver::Driver};

pub struct Publisher<'a, 'r, Config>
where
    Config: ZConfig,
{
    driver: &'a Driver<'r, Config>,
    ke: &'a keyexpr,

    encoding: Encoding<'a>,
    attachment: Option<Attachment<'a>>,
}

impl<'a, 'r, Config> Publisher<'a, 'r, Config>
where
    Config: ZConfig,
{
    pub fn keyexpr(&self) -> &keyexpr {
        self.ke
    }

    pub fn put(&'a self, payload: &'a [u8]) -> PublisherPutBuilder<'a, 'r, Config> {
        PublisherPutBuilder {
            driver: self.driver,
            ke: self.ke,
            payload,
            encoding: self.encoding.clone(),
            timestamp: None,
            attachment: self.attachment.clone(),
        }
    }
}

pub struct PublisherBuilder<'a, 'r, Config>
where
    Config: ZConfig,
{
    driver: &'a Driver<'r, Config>,

    ke: &'a keyexpr,

    encoding: Encoding<'a>,
    attachment: Option<Attachment<'a>>,
}

impl<'a, 'r, Config> PublisherBuilder<'a, 'r, Config>
where
    Config: ZConfig,
{
    pub(crate) fn new(driver: &'a Driver<'r, Config>, ke: &'a keyexpr) -> Self {
        Self {
            driver,
            ke,
            encoding: Encoding::bytes(),
            attachment: None,
        }
    }

    pub fn encoding(mut self, encoding: Encoding<'a>) -> Self {
        self.encoding = encoding;
        self
    }

    pub fn attachment(mut self, attachment: &'a [u8]) -> Self {
        self.attachment = Some(Attachment { buffer: attachment });
        self
    }

    pub async fn finish(self) -> crate::ZResult<Publisher<'a, 'r, Config>> {
        // TODO: send Publisher Interest message

        Ok(Publisher {
            driver: self.driver,
            ke: self.ke,
            encoding: self.encoding,
            attachment: self.attachment,
        })
    }
}

impl<'r, Config> super::Session<'r, Config>
where
    Config: ZConfig,
{
    pub fn declare_publisher<'a>(&'a self, ke: &'a keyexpr) -> PublisherBuilder<'a, 'r, Config> {
        PublisherBuilder::new(self.driver, ke)
    }
}

pub struct PublisherPutBuilder<'a, 'r, Config>
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

impl<'a, 'r, Config> PublisherPutBuilder<'a, 'r, Config>
where
    Config: ZConfig,
{
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
