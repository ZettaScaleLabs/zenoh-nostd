use zenoh_proto::{
    SessionError,
    exts::Attachment,
    fields::{Encoding, Timestamp},
    keyexpr,
};

use crate::{
    api::session::{Session, put::PutBuilder},
    config::ZSessionConfig,
};

pub struct Publisher<'parameters, 'session, 'ext, 'res, Config>
where
    Config: ZSessionConfig,
{
    session: &'session Session<'ext, 'res, Config>,

    ke: &'parameters keyexpr,

    encoding: Encoding<'parameters>,
    timestamp: Option<Timestamp>,
    attachment: Option<Attachment<'parameters>>,
}

impl<'parameters, 'session, 'ext, 'res, Config> Publisher<'parameters, 'session, 'ext, 'res, Config>
where
    Config: ZSessionConfig,
{
    pub fn put(
        &self,
        payload: &'parameters [u8],
    ) -> PutBuilder<'parameters, 'session, 'ext, 'res, Config> {
        PutBuilder {
            session: self.session,
            ke: self.ke,
            payload,
            encoding: self.encoding.clone(),
            timestamp: self.timestamp,
            attachment: self.attachment.clone(),
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

pub struct PublisherBuilder<'parameters, 'session, 'ext, 'res, Config>
where
    Config: ZSessionConfig,
{
    session: &'session Session<'ext, 'res, Config>,

    ke: &'parameters keyexpr,
    encoding: Encoding<'parameters>,
    timestamp: Option<Timestamp>,
    attachment: Option<Attachment<'parameters>>,
}

impl<'parameters, 'session, 'ext, 'res, Config>
    PublisherBuilder<'parameters, 'session, 'ext, 'res, Config>
where
    Config: ZSessionConfig,
{
    pub(crate) fn new(
        session: &'session Session<'ext, 'res, Config>,
        ke: &'parameters keyexpr,
    ) -> Self {
        Self {
            session,
            ke,
            encoding: Encoding::default(),
            timestamp: None,
            attachment: None,
        }
    }

    pub fn keyexpr(mut self, ke: &'parameters keyexpr) -> Self {
        self.ke = ke;
        self
    }

    pub fn encoding(mut self, encoding: Encoding<'parameters>) -> Self {
        self.encoding = encoding;
        self
    }

    pub fn timestamp(mut self, timestamp: Timestamp) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    pub fn attachment(mut self, attachment: &'parameters [u8]) -> Self {
        self.attachment = Some(Attachment { buffer: attachment });
        self
    }

    pub async fn finish(
        self,
    ) -> core::result::Result<Publisher<'parameters, 'session, 'ext, 'res, Config>, SessionError>
    {
        // TODO: send interest msg
        Ok(Publisher {
            session: self.session,
            ke: self.ke,
            encoding: self.encoding,
            timestamp: self.timestamp,
            attachment: self.attachment,
        })
    }
}

impl<'ext, 'res, Config> Session<'ext, 'res, Config>
where
    Config: ZSessionConfig,
{
    pub fn declare_publisher<'parameters>(
        &self,
        ke: &'parameters keyexpr,
    ) -> PublisherBuilder<'parameters, '_, 'ext, 'res, Config> {
        PublisherBuilder::new(self, ke)
    }
}
