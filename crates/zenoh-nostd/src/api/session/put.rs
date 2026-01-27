use zenoh_proto::{exts::*, fields::*, msgs::*, *};

use crate::{api::session::Session, config::ZSessionConfig, io::transport::ZTransportLinkTx};

pub struct PutBuilder<'parameters, 'session, 'ext, 'res, Config>
where
    Config: ZSessionConfig,
{
    pub(crate) session: &'session Session<'ext, 'res, Config>,

    pub(crate) ke: &'parameters keyexpr,
    pub(crate) payload: &'parameters [u8],

    pub(crate) encoding: Encoding<'parameters>,
    pub(crate) timestamp: Option<Timestamp>,
    pub(crate) attachment: Option<Attachment<'parameters>>,
}

impl<'parameters, 'session, 'ext, 'res, Config>
    PutBuilder<'parameters, 'session, 'ext, 'res, Config>
where
    Config: ZSessionConfig,
{
    pub(crate) fn new(
        session: &'session Session<'ext, 'res, Config>,
        ke: &'parameters keyexpr,
        payload: &'parameters [u8],
    ) -> Self {
        Self {
            session,
            ke,
            payload,
            encoding: Encoding::default(),
            timestamp: None,
            attachment: None,
        }
    }

    pub fn payload(mut self, payload: &'parameters [u8]) -> Self {
        self.payload = payload;
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

    pub async fn finish(self) -> core::result::Result<(), SessionError> {
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

        Ok(self
            .session
            .driver
            .tx()
            .await
            .send(core::iter::once(NetworkMessage {
                reliability: Reliability::default(),
                qos: QoS::default(),
                body: NetworkBody::Push(msg),
            }))
            .await?)
    }
}

impl<'ext, 'res, Config> Session<'ext, 'res, Config>
where
    Config: ZSessionConfig,
{
    pub fn put<'parameters>(
        &self,
        ke: &'parameters keyexpr,
        payload: &'parameters [u8],
    ) -> PutBuilder<'parameters, '_, 'ext, 'res, Config> {
        PutBuilder::new(self, ke, payload)
    }
}
