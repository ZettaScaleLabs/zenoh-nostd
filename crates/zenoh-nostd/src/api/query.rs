use core::str::FromStr;

use zenoh_proto::{CollectionError, SessionError, keyexpr};

use crate::{api::session::Session, config::ZSessionConfig};

pub struct QueryableQuery<'a, 'res, Config>
where
    Config: ZSessionConfig,
{
    session: &'a Session<'res, Config>,
    rid: u32,
    ke: &'a keyexpr,
    parameters: Option<&'a str>,
    payload: Option<&'a [u8]>,
}

impl<'a, 'res, Config> QueryableQuery<'a, 'res, Config>
where
    Config: ZSessionConfig,
{
    pub(crate) fn new(
        session: &'a Session<'res, Config>,
        rid: u32,
        ke: &'a keyexpr,
        parameters: Option<&'a str>,
        payload: Option<&'a [u8]>,
    ) -> Self {
        Self {
            session,
            rid,
            ke,
            parameters,
            payload,
        }
    }

    pub fn keyexpr(&self) -> &keyexpr {
        self.ke
    }

    pub fn parameters(&self) -> Option<&str> {
        self.parameters
    }

    pub fn payload(&self) -> Option<&[u8]> {
        self.payload
    }

    pub async fn reply(
        &self,
        ke: &keyexpr,
        payload: &[u8],
    ) -> core::result::Result<(), SessionError> {
        self.session.reply(self.rid, ke, payload).await
    }

    pub async fn err(
        &self,
        ke: &keyexpr,
        payload: &[u8],
    ) -> core::result::Result<(), SessionError> {
        self.session.err(self.rid, ke, payload).await
    }

    pub async fn finalize(&self) -> core::result::Result<(), SessionError> {
        self.session.finalize(self.rid).await
    }
}

pub struct OwnedQueryableQuery<
    Config,
    const MAX_KEYEXPR: usize,
    const MAX_PARAMETERS: usize,
    const MAX_PAYLOAD: usize,
> where
    Config: ZSessionConfig + 'static,
{
    session: &'static Session<'static, Config>,
    rid: u32,
    ke: heapless::String<MAX_KEYEXPR>,
    parameters: Option<heapless::String<MAX_PARAMETERS>>,
    payload: Option<heapless::Vec<u8, MAX_PAYLOAD>>,
}

impl<Config, const MAX_KEYEXPR: usize, const MAX_PARAMETERS: usize, const MAX_PAYLOAD: usize>
    OwnedQueryableQuery<Config, MAX_KEYEXPR, MAX_PARAMETERS, MAX_PAYLOAD>
where
    Config: ZSessionConfig,
{
    pub fn keyexpr(&self) -> &keyexpr {
        keyexpr::from_str_unchecked(self.ke.as_str())
    }

    pub fn parameters(&self) -> Option<&str> {
        self.parameters.as_ref().map(|p| p.as_str())
    }

    pub fn payload(&self) -> Option<&[u8]> {
        self.payload.as_ref().map(|p| p.as_slice())
    }

    pub async fn reply(
        &self,
        ke: &keyexpr,
        payload: &[u8],
    ) -> core::result::Result<(), SessionError> {
        self.session.reply(self.rid, ke, payload).await
    }

    pub async fn err(
        &self,
        ke: &keyexpr,
        payload: &[u8],
    ) -> core::result::Result<(), SessionError> {
        self.session.err(self.rid, ke, payload).await
    }

    pub async fn finalize(&self) -> core::result::Result<(), SessionError> {
        self.session.finalize(self.rid).await
    }
}

impl<'a, Config, const MAX_KEYEXPR: usize, const MAX_PARAMETERS: usize, const MAX_PAYLOAD: usize>
    TryFrom<(
        &QueryableQuery<'a, 'static, Config>,
        &'static Session<'static, Config>,
    )> for OwnedQueryableQuery<Config, MAX_KEYEXPR, MAX_PARAMETERS, MAX_PAYLOAD>
where
    Config: ZSessionConfig,
{
    type Error = CollectionError;

    fn try_from(
        value: (
            &QueryableQuery<'a, 'static, Config>,
            &'static Session<'static, Config>,
        ),
    ) -> Result<Self, Self::Error> {
        let (value, session) = value;

        Ok(Self {
            session: session,
            rid: value.rid,
            ke: heapless::String::from_str(value.keyexpr().as_str())
                .map_err(|_| CollectionError::CollectionTooSmall)?,
            parameters: value
                .parameters
                .map(heapless::String::from_str)
                .transpose()
                .map_err(|_| CollectionError::CollectionTooSmall)?,
            payload: value
                .payload
                .map(heapless::Vec::from_slice)
                .transpose()
                .map_err(|_| CollectionError::CollectionTooSmall)?,
        })
    }
}
