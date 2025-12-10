use zenoh_proto::{exts::*, fields::*, *};

use crate::api::{ZConfig, driver::Driver, session::put::PutBuilder};

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

    pub fn put(&'a self, payload: &'a [u8]) -> PutBuilder<'a, 'r, Config> {
        let mut put =
            PutBuilder::new(self.driver, self.ke, payload).encoding(self.encoding.clone());

        if let Some(attachment) = self.attachment.clone() {
            put = put.attachment(attachment.buffer);
        }

        put
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
