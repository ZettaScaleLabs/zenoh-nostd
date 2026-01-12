use zenoh_proto::{
    exts::Attachment,
    fields::{Encoding, Timestamp},
    keyexpr,
};

use crate::api::{ZConfig, driver::Driver, session::put::PutBuilder};

pub struct Publisher<'a, 'res, Config>
where
    Config: ZConfig,
{
    driver: &'a Driver<'res, Config>,

    ke: &'a keyexpr,
    encoding: Encoding<'a>,
    timestamp: Option<Timestamp>,
    attachment: Option<Attachment<'a>>,
}

impl<'a, 'res, Config> Publisher<'a, 'res, Config>
where
    Config: ZConfig,
{
    pub fn put(&self, payload: &'a [u8]) -> PutBuilder<'a, 'res, Config> {
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

pub struct PublisherBuilder<'a, 'res, Config>
where
    Config: ZConfig,
{
    driver: &'a Driver<'res, Config>,

    ke: &'a keyexpr,
    encoding: Encoding<'a>,
    timestamp: Option<Timestamp>,
    attachment: Option<Attachment<'a>>,
}

impl<'a, 'res, Config> PublisherBuilder<'a, 'res, Config>
where
    Config: ZConfig,
{
    pub(crate) fn new(driver: &'a Driver<'res, Config>, ke: &'a keyexpr) -> Self {
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

    pub async fn finish(self) -> crate::ZResult<Publisher<'a, 'res, Config>> {
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

impl<'res, Config> super::Session<'res, Config>
where
    Config: ZConfig,
{
    pub fn declare_publisher<'a>(&'a self, ke: &'a keyexpr) -> PublisherBuilder<'a, 'res, Config> {
        PublisherBuilder::new(&self.driver, ke)
    }
}
