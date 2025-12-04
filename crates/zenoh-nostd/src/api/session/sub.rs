use embassy_sync::channel::DynamicReceiver;
use zenoh_proto::{fields::*, msgs::*, *};

use crate::{
    api::{CallbackId, OwnedSample, SessionResources, driver::Driver},
    platform::ZPlatform,
};

pub struct Subscriber<'a, const MAX_KEYEXPR_LEN: usize, const MAX_PAYLOAD_LEN: usize> {
    id: u32,
    ke: &'static keyexpr,
    inner: Option<DynamicReceiver<'a, OwnedSample<MAX_KEYEXPR_LEN, MAX_PAYLOAD_LEN>>>,
}

impl<'a, const MAX_KEYEXPR_LEN: usize, const MAX_PAYLOAD_LEN: usize>
    Subscriber<'a, MAX_KEYEXPR_LEN, MAX_PAYLOAD_LEN>
{
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn keyexpr(&self) -> &'static keyexpr {
        self.ke
    }

    pub async fn recv(&mut self) -> Option<OwnedSample<MAX_KEYEXPR_LEN, MAX_PAYLOAD_LEN>> {
        match &mut self.inner {
            Some(receiver) => Some(receiver.receive().await),
            None => None,
        }
    }
}

pub struct DeclareSubscriberBuilder<'a, 'b, Platform, TxBuf, RxBuf, SessionResources>
where
    Platform: ZPlatform,
{
    driver: &'b Driver<'a, Platform, TxBuf, RxBuf>,
    resources: &'b SessionResources,

    ke: &'static keyexpr,
    callback: CallbackId,
}

impl<
    'a,
    'b,
    Platform,
    TxBuf,
    RxBuf,
    const MAX_KEYEXPR_LEN: usize,
    const MAX_PARAMETERS_LEN: usize,
    const MAX_PAYLOAD_LEN: usize,
    const MAX_QUEUED: usize,
    const MAX_CALLBACKS: usize,
    const MAX_SUBSCRIBERS: usize,
>
    DeclareSubscriberBuilder<
        'a,
        'b,
        Platform,
        TxBuf,
        RxBuf,
        SessionResources<
            MAX_KEYEXPR_LEN,
            MAX_PARAMETERS_LEN,
            MAX_PAYLOAD_LEN,
            MAX_QUEUED,
            MAX_CALLBACKS,
            MAX_SUBSCRIBERS,
        >,
    >
where
    Platform: ZPlatform,
{
    pub(crate) fn new(
        driver: &'b Driver<'a, Platform, TxBuf, RxBuf>,
        resources: &'b SessionResources<
            MAX_KEYEXPR_LEN,
            MAX_PARAMETERS_LEN,
            MAX_PAYLOAD_LEN,
            MAX_QUEUED,
            MAX_CALLBACKS,
            MAX_SUBSCRIBERS,
        >,
        ke: &'static keyexpr,
        callback: CallbackId,
    ) -> Self {
        Self {
            driver,
            resources,
            ke,
            callback,
        }
    }
}

impl<
    'a,
    'b,
    Platform,
    TxBuf,
    RxBuf,
    const MAX_KEYEXPR_LEN: usize,
    const MAX_PARAMETERS_LEN: usize,
    const MAX_PAYLOAD_LEN: usize,
    const MAX_QUEUED: usize,
    const MAX_CALLBACKS: usize,
    const MAX_SUBSCRIBERS: usize,
>
    DeclareSubscriberBuilder<
        'a,
        'b,
        Platform,
        TxBuf,
        RxBuf,
        SessionResources<
            MAX_KEYEXPR_LEN,
            MAX_PARAMETERS_LEN,
            MAX_PAYLOAD_LEN,
            MAX_QUEUED,
            MAX_CALLBACKS,
            MAX_SUBSCRIBERS,
        >,
    >
where
    Platform: ZPlatform,
    TxBuf: AsMut<[u8]>,
{
    pub async fn finish(self) -> crate::ZResult<Subscriber<'b, MAX_KEYEXPR_LEN, MAX_PAYLOAD_LEN>> {
        let id = self
            .resources
            .register_subscriber(self.ke, self.callback)
            .await?;

        let msg = Declare {
            body: DeclareBody::DeclareSubscriber(DeclareSubscriber {
                id: id,
                wire_expr: WireExpr::from(self.ke),
            }),
            ..Default::default()
        };

        self.driver.send(msg).await?;

        Ok(Subscriber {
            id,
            ke: self.ke,
            inner: self.resources.subscriber_receiver(self.callback),
        })
    }
}

impl<
    'a,
    Platform,
    TxBuf,
    RxBuf,
    const MAX_KEYEXPR_LEN: usize,
    const MAX_PARAMETERS_LEN: usize,
    const MAX_PAYLOAD_LEN: usize,
    const MAX_QUEUED: usize,
    const MAX_CALLBACKS: usize,
    const MAX_SUBSCRIBERS: usize,
>
    super::PublicSession<
        'a,
        Platform,
        TxBuf,
        RxBuf,
        SessionResources<
            MAX_KEYEXPR_LEN,
            MAX_PARAMETERS_LEN,
            MAX_PAYLOAD_LEN,
            MAX_QUEUED,
            MAX_CALLBACKS,
            MAX_SUBSCRIBERS,
        >,
    >
where
    Platform: ZPlatform,
    TxBuf: AsMut<[u8]>,
{
    pub fn declare_subscriber<'b>(
        &'b self,
        ke: &'static keyexpr,
        callback: CallbackId,
    ) -> DeclareSubscriberBuilder<
        'a,
        'b,
        Platform,
        TxBuf,
        RxBuf,
        SessionResources<
            MAX_KEYEXPR_LEN,
            MAX_PARAMETERS_LEN,
            MAX_PAYLOAD_LEN,
            MAX_QUEUED,
            MAX_CALLBACKS,
            MAX_SUBSCRIBERS,
        >,
    > {
        DeclareSubscriberBuilder::new(&self.driver, &self.resources, ke, callback)
    }
}
