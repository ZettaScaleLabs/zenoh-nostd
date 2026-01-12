use embassy_time::Duration;
use zenoh_proto::keyexpr;

use crate::api::{ZConfig, driver::Driver, resources::SessionResources, session::get::GetBuilder};

pub struct Querier<'this, 'res, Config>
where
    Config: ZConfig,
{
    driver: &'this Driver<'this, Config>,
    resources: &'this SessionResources<'res, Config>,

    ke: &'static keyexpr,
    parameters: Option<&'this str>,
    payload: Option<&'this [u8]>,
    timeout: Option<Duration>,
}

impl<'this, 'res, Config> Querier<'this, 'res, Config>
where
    Config: ZConfig,
{
    pub fn get(&self) -> GetBuilder<'this, 'res, Config> {
        GetBuilder {
            driver: self.driver,
            resources: self.resources,
            ke: self.ke,
            parameters: self.parameters.clone(),
            payload: self.payload.clone(),
            timeout: self.timeout.clone(),
            callback: None,
            receiver: None,
        }
    }

    pub async fn undeclare(self) -> crate::ZResult<()> {
        todo!()
    }

    pub fn keyexpr(&self) -> &keyexpr {
        self.ke
    }
}

pub struct QuerierBuilder<'this, 'res, Config>
where
    Config: ZConfig,
{
    driver: &'this Driver<'this, Config>,
    resources: &'this SessionResources<'res, Config>,

    ke: &'static keyexpr,
    parameters: Option<&'this str>,
    payload: Option<&'this [u8]>,
    timeout: Option<Duration>,
}

impl<'this, 'res, Config> QuerierBuilder<'this, 'res, Config>
where
    Config: ZConfig,
{
    pub(crate) fn new(
        driver: &'this Driver<'this, Config>,
        resources: &'this SessionResources<'res, Config>,
        ke: &'static keyexpr,
    ) -> Self {
        Self {
            driver,
            resources,
            ke,
            parameters: None,
            payload: None,
            timeout: None,
        }
    }

    pub fn parameters(mut self, parameters: &'this str) -> Self {
        self.parameters = Some(parameters);
        self
    }

    pub fn payload(mut self, payload: &'this [u8]) -> Self {
        self.payload = Some(payload);
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub async fn finish(self) -> crate::ZResult<Querier<'this, 'res, Config>> {
        // TODO: send interest msg
        Ok(Querier {
            driver: self.driver,
            resources: self.resources,
            ke: self.ke,
            parameters: self.parameters,
            payload: self.payload,
            timeout: self.timeout,
        })
    }
}

impl<'this, 'res, Config> super::Session<'this, 'res, Config>
where
    Config: ZConfig,
{
    pub fn declare_querier(&self, ke: &'static keyexpr) -> QuerierBuilder<'this, 'res, Config> {
        QuerierBuilder::new(self.driver, self.resources, ke)
    }
}
