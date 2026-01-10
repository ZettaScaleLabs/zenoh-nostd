use zenoh_proto::keyexpr;

use crate::api::{ZConfig, driver::Driver, session::put::PutBuilder};

pub struct Publisher<'this, Config>
where
    Config: ZConfig,
{
    driver: &'this Driver<'this, Config>,
    ke: &'static keyexpr,
}

impl<'this, Config> Publisher<'this, Config>
where
    Config: ZConfig,
{
    pub async fn put<'a>(&self, payload: &'a [u8]) -> PutBuilder<'this, 'a, Config> {
        PutBuilder::new(self.driver, self.ke, payload)
    }

    pub async fn undeclare(self) -> crate::ZResult<()> {
        todo!()
    }
}

pub struct PublisherBuilder<'this, Config>
where
    Config: ZConfig,
{
    driver: &'this Driver<'this, Config>,

    ke: &'static keyexpr,
}

impl<'this, Config> PublisherBuilder<'this, Config>
where
    Config: ZConfig,
{
    pub(crate) fn new(driver: &'this Driver<'this, Config>, ke: &'static keyexpr) -> Self {
        Self { driver, ke }
    }
}

impl<'this, Config> PublisherBuilder<'this, Config>
where
    Config: ZConfig,
{
    pub async fn finish(self) -> crate::ZResult<Publisher<'this, Config>> {
        Ok(Publisher {
            driver: self.driver,
            ke: self.ke,
        })
    }
}

impl<'this, 'res, Config> super::Session<'this, 'res, Config>
where
    Config: ZConfig,
{
    pub fn declare_publisher(&self, ke: &'static keyexpr) -> PublisherBuilder<'this, Config> {
        PublisherBuilder::new(self.driver, ke)
    }
}
