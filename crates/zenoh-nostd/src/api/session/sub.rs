use core::fmt::Display;

use dyn_utils::DynObject;
use embassy_sync::channel::{DynamicReceiver, DynamicSender};
use zenoh_proto::{fields::*, msgs::*, *};

use crate::api::{
    ZConfig,
    arg::SampleRef,
    callbacks::{AsyncCallback, DynCallback, SyncCallback, ZCallbacks},
    driver::Driver,
    resources::SessionResources,
};

pub struct Subscriber<'a, 'res, Config, OwnedSample = (), const CHANNEL: bool = false>
where
    Config: ZConfig,
{
    id: u32,

    driver: &'a Driver<'res, Config>,
    resources: &'a SessionResources<'res, Config>,

    receiver: Option<DynamicReceiver<'res, OwnedSample>>,
}

impl<'a, 'res, Config, OwnedSample, const CHANNEL: bool>
    Subscriber<'a, 'res, Config, OwnedSample, CHANNEL>
where
    Config: ZConfig,
{
    pub async fn undeclare(self) -> crate::ZResult<()> {
        let msg = Declare {
            body: DeclareBody::UndeclareSubscriber(UndeclareSubscriber {
                id: self.id,
                ..Default::default()
            }),
            ..Default::default()
        };

        self.resources.sub_callbacks.lock().await.remove(self.id)?;

        self.driver.send(msg).await?;

        todo!("Also stop the channel if any")
    }
}

impl<'a, 'res, Config, OwnedSample> Subscriber<'a, 'res, Config, OwnedSample, true>
where
    Config: ZConfig,
{
    pub fn try_recv(&self) -> Option<OwnedSample> {
        self.receiver.as_ref().unwrap().try_receive().ok()
    }

    pub async fn recv(&self) -> Option<OwnedSample> {
        Some(self.receiver.as_ref().unwrap().receive().await)
    }
}

pub struct SubscriberBuilder<
    'a,
    'res,
    Config,
    OwnedSample = (),
    const READY: bool = false,
    const CHANNEL: bool = false,
> where
    Config: ZConfig,
{
    driver: &'a Driver<'res, Config>,
    resources: &'a SessionResources<'res, Config>,

    ke: &'static keyexpr,

    callback: Option<
        DynCallback<
            'res,
            <Config::SubCallbacks<'res> as ZCallbacks<'res, SampleRef>>::Callback,
            <Config::SubCallbacks<'res> as ZCallbacks<'res, SampleRef>>::Future,
            SampleRef,
        >,
    >,
    receiver: Option<DynamicReceiver<'res, OwnedSample>>,
}

impl<'a, 'res, Config> SubscriberBuilder<'a, 'res, Config, (), false, false>
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
            callback: None,
            receiver: None,
        }
    }

    pub fn callback(
        self,
        callback: impl AsyncFnMut(&crate::Sample<'_>) + 'res,
    ) -> SubscriberBuilder<'a, 'res, Config, (), true, false> {
        SubscriberBuilder {
            driver: self.driver,
            resources: self.resources,
            ke: self.ke,
            callback: Some(DynObject::new(AsyncCallback::new(callback))),
            receiver: None,
        }
    }

    pub fn callback_sync(
        self,
        callback: impl FnMut(&crate::Sample<'_>) + 'res,
    ) -> SubscriberBuilder<'a, 'res, Config, (), true, false> {
        SubscriberBuilder {
            driver: self.driver,
            resources: self.resources,
            ke: self.ke,
            callback: Some(DynObject::new(SyncCallback::new(callback))),
            receiver: None,
        }
    }

    pub fn channel<OwnedSample, E>(
        self,
        sender: DynamicSender<'res, OwnedSample>,
        receiver: DynamicReceiver<'res, OwnedSample>,
    ) -> SubscriberBuilder<'a, 'res, Config, OwnedSample, true, true>
    where
        OwnedSample: for<'any> TryFrom<&'any crate::Sample<'any>, Error = E>,
        E: Display,
    {
        SubscriberBuilder {
            driver: self.driver,
            resources: self.resources,
            ke: self.ke,
            callback: Some(DynObject::new(AsyncCallback::new(
                async move |resp: &'_ crate::Sample<'_>| {
                    let resp = OwnedSample::try_from(resp);
                    match resp {
                        Ok(resp) => {
                            sender.send(resp).await;
                        }
                        Err(e) => {
                            crate::error!("{}: {}", crate::zctx!(), e)
                        }
                    }
                },
            ))),
            receiver: Some(receiver),
        }
    }
}

impl<'a, 'res, Config, OwnedSample, const CHANNEL: bool>
    SubscriberBuilder<'a, 'res, Config, OwnedSample, true, CHANNEL>
where
    Config: ZConfig,
{
    pub async fn finish(
        self,
    ) -> crate::ZResult<Subscriber<'a, 'res, Config, OwnedSample, CHANNEL>> {
        let id = self.resources.next().await;

        if let Some(callback) = self.callback {
            let mut subs = self.resources.sub_callbacks.lock().await;
            subs.drop_timedout();
            subs.insert(id, self.ke, None, callback)?;
        }

        let msg = Declare {
            body: DeclareBody::DeclareSubscriber(DeclareSubscriber {
                id,
                wire_expr: WireExpr::from(self.ke),
            }),
            ..Default::default()
        };

        self.driver.send(msg).await?;

        Ok(Subscriber {
            id,
            driver: self.driver,
            resources: self.resources,
            receiver: self.receiver,
        })
    }
}

impl<'res, Config> super::Session<'res, Config>
where
    Config: ZConfig,
{
    pub fn declare_subscriber(&self, ke: &'static keyexpr) -> SubscriberBuilder<'_, 'res, Config> {
        SubscriberBuilder::new(&self.driver, &self.resources, ke)
    }
}
