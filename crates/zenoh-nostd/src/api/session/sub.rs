use crate::api::{
    HeaplessCallbacks, HeaplessChannels, HeaplessSample, SamplePtr, SessionResources, ZCallbacks,
    ZChannel, ZChannels, ZConfig, driver::Driver,
};
use zenoh_proto::{fields::*, keyexpr, msgs::*};

pub type HeaplessSubscriberCallbacks<const CALLBACK_MEMORY: usize, const CAPACITY: usize> =
    HeaplessCallbacks<SamplePtr, (), CALLBACK_MEMORY, CAPACITY>;

pub type HeaplessSubscriberChannels<
    const MAX_KEYEXPR: usize,
    const MAX_PAYLOAD: usize,
    const QUEUED: usize,
    const CAPACITY: usize,
> = HeaplessChannels<HeaplessSample<MAX_KEYEXPR, MAX_PAYLOAD>, QUEUED, CAPACITY>;

pub struct Subscriber<'a, 'r, Config>
where
    Config: ZConfig,
{
    driver: &'a Driver<'r, Config>,
    resources: &'r SessionResources<Config>,

    ke: &'static keyexpr,
    id: u32,

    channel: Option<&'r <Config::SubscriberChannels as ZChannels<SamplePtr>>::Channel>,
}

impl<'r, Config> Subscriber<'_, 'r, Config>
where
    Config: ZConfig,
{
    pub fn keyexpr(&self) -> &'static keyexpr {
        self.ke
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub async fn recv(
        &self,
    ) -> core::result::Result<
        <<Config::SubscriberChannels as ZChannels<SamplePtr>>::Channel as ZChannel<SamplePtr>>::Item,
    crate::SubscriberError,>{
        if let Some(ch) = &self.channel {
            Ok(ch.recv().await)
        } else {
            Err(crate::SubscriberError::SubscriberChannelClosed)
        }
    }

    pub async fn undeclare(self) -> crate::ZResult<()> {
        let msg = Declare {
            body: DeclareBody::UndeclareSubscriber(UndeclareSubscriber {
                id: self.id,
                ..Default::default()
            }),
            ..Default::default()
        };

        self.resources.sub_callbacks.lock().await.remove(self.id)?;
        self.resources.sub_channels.remove(self.id).await?;

        self.driver.send(msg).await?;

        Ok(())
    }
}

pub struct SubscriberBuilder<'a, 'r, Config>
where
    Config: ZConfig,
{
    driver: &'a Driver<'r, Config>,
    resources: &'r SessionResources<Config>,

    ke: &'static keyexpr,

    callback: Option<<Config::SubscriberCallbacks as ZCallbacks<SamplePtr, ()>>::Callback>,
}

impl<'a, 'r, Config> SubscriberBuilder<'a, 'r, Config>
where
    Config: ZConfig,
{
    pub(crate) fn new(
        driver: &'a Driver<'r, Config>,
        resources: &'r SessionResources<Config>,
        ke: &'static keyexpr,
    ) -> Self {
        Self {
            driver,
            resources,
            ke,
            callback: None,
        }
    }

    pub fn callback(
        mut self,
        callback: <Config::SubscriberCallbacks as ZCallbacks<SamplePtr, ()>>::Callback,
    ) -> Self {
        self.callback = Some(callback);
        self
    }

    pub async fn finish(self) -> crate::ZResult<Subscriber<'a, 'r, Config>> {
        let id = self.resources.next().await;

        let mut subscribers = self.resources.sub_callbacks.lock().await;
        let channel = if let Some(callback) = self.callback {
            subscribers.insert(id, self.ke, callback)?;
            None
        } else {
            let channels = &self.resources.sub_channels;
            let ch = channels.insert(id, self.ke).await?;
            Some(ch)
        };

        let msg = Declare {
            body: DeclareBody::DeclareSubscriber(DeclareSubscriber {
                id,
                wire_expr: WireExpr::from(self.ke),
            }),
            ..Default::default()
        };

        self.driver.send(msg).await?;

        Ok(Subscriber {
            driver: self.driver,
            resources: self.resources,
            ke: self.ke,
            id,
            channel,
        })
    }
}

impl<'r, Config> super::Session<'r, Config>
where
    Config: ZConfig,
{
    pub fn declare_subscriber<'a>(
        &'a self,
        ke: &'static keyexpr,
    ) -> SubscriberBuilder<'a, 'r, Config> {
        SubscriberBuilder::new(self.driver, self.resources, ke)
    }
}
