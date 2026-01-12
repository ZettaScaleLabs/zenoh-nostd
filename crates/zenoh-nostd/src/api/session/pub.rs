use zenoh_proto::{
    exts::Attachment,
    fields::{Encoding, Timestamp},
    keyexpr,
};

use crate::api::{ZConfig, driver::Driver, session::put::PutBuilder};

pub struct Publisher<'this, 'a, Config>
where
    Config: ZConfig,
{
    driver: &'this Driver<'this, Config>,
    ke: &'a keyexpr,
    encoding: Encoding<'a>,
    timestamp: Option<Timestamp>,
    attachment: Option<Attachment<'a>>,
}

impl<'this, 'a, Config> Publisher<'this, 'a, Config>
where
    Config: ZConfig,
{
    pub fn put(&self, payload: &'a [u8]) -> PutBuilder<'this, 'a, Config> {
        PutBuilder {
            driver: self.driver,
            ke: self.ke,
            payload: payload,
            encoding: self.encoding.clone(),
            timestamp: self.timestamp.clone(),
            attachment: self.attachment.clone(),
        }
    }

    pub async fn undeclare(self) -> crate::ZResult<()> {
        todo!()
    }

    pub fn keyexpr(&self) -> &keyexpr {
        self.ke
    }
}

pub struct PublisherBuilder<'this, 'a, Config>
where
    Config: ZConfig,
{
    driver: &'this Driver<'this, Config>,

    ke: &'a keyexpr,
    encoding: Encoding<'a>,
    timestamp: Option<Timestamp>,
    attachment: Option<Attachment<'a>>,
}

impl<'this, 'a, Config> PublisherBuilder<'this, 'a, Config>
where
    Config: ZConfig,
{
    pub(crate) fn new(driver: &'this Driver<'this, Config>, ke: &'a keyexpr) -> Self {
        Self {
            driver,
            ke,
            encoding: Encoding::default(),
            timestamp: None,
            attachment: None,
        }
    }

    pub fn keyexpr(mut self, ke: &'a keyexpr) -> Self {
        self.ke = ke;
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

    pub async fn finish(self) -> crate::ZResult<Publisher<'this, 'a, Config>> {
        // TODO: send interest msg

        Ok(Publisher {
            driver: self.driver,
            ke: self.ke,
            encoding: self.encoding,
            timestamp: self.timestamp,
            attachment: self.attachment,
        })
    }
}

impl<'this, 'res, Config> super::Session<'this, 'res, Config>
where
    Config: ZConfig,
{
    pub fn declare_publisher<'a>(&self, ke: &'a keyexpr) -> PublisherBuilder<'this, 'a, Config> {
        PublisherBuilder::new(self.driver, ke)
    }
}
