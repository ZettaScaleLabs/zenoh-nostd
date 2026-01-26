use core::time::Duration;
use zenoh_proto::{SessionError, keyexpr};

use crate::{api::session::Session, config::ZSessionConfig, session::GetBuilder};

pub struct Querier<'parameters, 'session, 'ext, 'res, Config>
where
    Config: ZSessionConfig,
{
    session: &'session Session<'ext, 'res, Config>,
    ke: &'static keyexpr,
    parameters: Option<&'parameters str>,
    payload: Option<&'parameters [u8]>,
    timeout: Option<Duration>,
}

impl<'parameters, 'session, 'ext, 'res, Config> Querier<'parameters, 'session, 'ext, 'res, Config>
where
    Config: ZSessionConfig,
{
    pub fn get(&self) -> GetBuilder<'parameters, 'session, 'ext, 'res, Config> {
        GetBuilder {
            session: self.session,
            ke: self.ke,
            parameters: self.parameters,
            payload: self.payload,
            timeout: self.timeout,
            callback: None,
            receiver: None,
        }
    }

    #[allow(dead_code)]
    async fn undeclare(self) -> core::result::Result<(), SessionError> {
        todo!("send undeclare interest")
    }

    pub fn keyexpr(&self) -> &keyexpr {
        self.ke
    }
}

pub struct QuerierBuilder<'parameters, 'session, 'ext, 'res, Config>
where
    Config: ZSessionConfig,
{
    session: &'session Session<'ext, 'res, Config>,
    ke: &'static keyexpr,
    parameters: Option<&'parameters str>,
    payload: Option<&'parameters [u8]>,
    timeout: Option<Duration>,
}

impl<'parameters, 'session, 'ext, 'res, Config>
    QuerierBuilder<'parameters, 'session, 'ext, 'res, Config>
where
    Config: ZSessionConfig,
{
    pub(crate) fn new(
        session: &'session Session<'ext, 'res, Config>,
        ke: &'static keyexpr,
    ) -> Self {
        Self {
            session,
            ke,
            parameters: None,
            payload: None,
            timeout: None,
        }
    }

    pub fn parameters(mut self, parameters: &'parameters str) -> Self {
        self.parameters = Some(parameters);
        self
    }

    pub fn payload(mut self, payload: &'parameters [u8]) -> Self {
        self.payload = Some(payload);
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub async fn finish(
        self,
    ) -> core::result::Result<Querier<'parameters, 'session, 'ext, 'res, Config>, SessionError>
    {
        // TODO: send interest msg
        Ok(Querier {
            session: self.session,
            ke: self.ke,
            parameters: self.parameters,
            payload: self.payload,
            timeout: self.timeout,
        })
    }
}

impl<'ext, 'res, Config> Session<'ext, 'res, Config>
where
    Config: ZSessionConfig,
{
    pub fn declare_querier<'parameters>(
        &self,
        ke: &'static keyexpr,
    ) -> QuerierBuilder<'parameters, '_, 'ext, 'res, Config> {
        QuerierBuilder::new(self, ke)
    }
}
