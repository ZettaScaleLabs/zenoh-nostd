use embassy_time::Duration;
use zenoh_proto::keyexpr;

use crate::api::{ZConfig, driver::Driver, resources::SessionResources, session::get::GetBuilder};

pub struct Querier<'a, 'res, Config>
where
    Config: ZConfig,
{
    driver: &'a Driver<'res, Config>,
    resources: &'a SessionResources<'res, Config>,

    ke: &'static keyexpr,
    parameters: Option<&'a str>,
    payload: Option<&'a [u8]>,
    timeout: Option<Duration>,
}

impl<'a, 'res, Config> Querier<'a, 'res, Config>
where
    Config: ZConfig,
{
    pub fn get(&self) -> GetBuilder<'a, 'res, Config> {
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
        todo!("send undeclare interest")
    }

    pub fn keyexpr(&self) -> &keyexpr {
        self.ke
    }
}

pub struct QuerierBuilder<'a, 'res, Config>
where
    Config: ZConfig,
{
    driver: &'a Driver<'res, Config>,
    resources: &'a SessionResources<'res, Config>,

    ke: &'static keyexpr,
    parameters: Option<&'a str>,
    payload: Option<&'a [u8]>,
    timeout: Option<Duration>,
}

impl<'a, 'res, Config> QuerierBuilder<'a, 'res, Config>
where
    Config: ZConfig,
{
    pub(crate) fn new(
        driver: &'a Driver<'res, Config>,
        resources: &'a SessionResources<'res, Config>,
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

    pub fn parameters(mut self, parameters: &'a str) -> Self {
        self.parameters = Some(parameters);
        self
    }

    pub fn payload(mut self, payload: &'a [u8]) -> Self {
        self.payload = Some(payload);
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub async fn finish(self) -> crate::ZResult<Querier<'a, 'res, Config>> {
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

impl<'res, Config> super::Session<'res, Config>
where
    Config: ZConfig,
{
    pub fn declare_querier(&self, ke: &'static keyexpr) -> QuerierBuilder<'_, 'res, Config> {
        QuerierBuilder::new(&self.driver, &self.resources, ke)
    }
}
