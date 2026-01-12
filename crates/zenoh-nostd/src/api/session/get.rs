use std::fmt::Display;

use dyn_utils::DynObject;
use embassy_futures::select::{Either, select};
use embassy_sync::channel::{DynamicReceiver, DynamicSender};
use embassy_time::{Duration, Instant, Timer};
use zenoh_proto::{exts::*, fields::*, msgs::*, *};

use crate::api::{
    ZConfig,
    arg::ResponseRef,
    callbacks::{AsyncCallback, DynCallback, SyncCallback, ZCallbacks},
    driver::Driver,
    resources::SessionResources,
};

pub struct Responses<'this, OwnedResponse = (), const CHANNEL: bool = false> {
    timedout: Instant,
    receiver: Option<DynamicReceiver<'this, OwnedResponse>>,
}

impl<'this, OwnedResponse> Responses<'this, OwnedResponse, true> {
    pub fn try_recv(&self) -> Option<OwnedResponse> {
        self.receiver.as_ref().unwrap().try_receive().ok()
    }

    pub async fn recv(&self) -> Option<OwnedResponse> {
        match select(
            Timer::at(self.timedout),
            self.receiver.as_ref().unwrap().receive(),
        )
        .await
        {
            Either::First(_) => None,
            Either::Second(v) => Some(v),
        }
    }
}

pub struct GetBuilder<
    'this,
    'res,
    Config,
    OwnedResponse = (),
    const READY: bool = false,
    const CHANNEL: bool = false,
> where
    Config: ZConfig,
{
    pub(crate) driver: &'this Driver<'this, Config>,
    pub(crate) resources: &'this SessionResources<'res, Config>,

    pub(crate) ke: &'static keyexpr,
    pub(crate) parameters: Option<&'this str>,
    pub(crate) payload: Option<&'this [u8]>,
    pub(crate) timeout: Option<Duration>,
    pub(crate) callback: Option<
        DynCallback<
            'res,
            <Config::GetCallbacks<'res> as ZCallbacks<'res, ResponseRef>>::Callback,
            <Config::GetCallbacks<'res> as ZCallbacks<'res, ResponseRef>>::Future,
            ResponseRef,
        >,
    >,
    pub(crate) receiver: Option<DynamicReceiver<'this, OwnedResponse>>,
}

impl<'this, 'res, Config> GetBuilder<'this, 'res, Config>
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
            callback: None,
            receiver: None,
        }
    }
}

impl<'this, 'res, Config> GetBuilder<'this, 'res, Config>
where
    Config: ZConfig,
{
    pub fn keyexpr(mut self, ke: &'static keyexpr) -> Self {
        self.ke = ke;
        self
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
}

impl<'this, 'res, Config> GetBuilder<'this, 'res, Config, (), false>
where
    Config: ZConfig,
{
    pub fn callback(
        self,
        callback: impl AsyncFnMut(&crate::Response<'_>) + 'res,
    ) -> GetBuilder<'this, 'res, Config, (), true> {
        GetBuilder {
            driver: self.driver,
            resources: self.resources,
            ke: self.ke,
            parameters: self.parameters,
            payload: self.payload,
            timeout: self.timeout,
            callback: Some(DynObject::new(AsyncCallback::new(callback))),
            receiver: None,
        }
    }

    pub fn callback_sync(
        self,
        callback: impl FnMut(&crate::Response<'_>) + 'res,
    ) -> GetBuilder<'this, 'res, Config, (), true> {
        GetBuilder {
            driver: self.driver,
            resources: self.resources,
            ke: self.ke,
            parameters: self.parameters,
            payload: self.payload,
            timeout: self.timeout,
            callback: Some(DynObject::new(SyncCallback::new(callback))),
            receiver: None,
        }
    }

    pub fn channel<OwnedResponse, E>(
        self,
        sender: DynamicSender<'res, OwnedResponse>,
        receiver: DynamicReceiver<'res, OwnedResponse>,
    ) -> GetBuilder<'this, 'res, Config, OwnedResponse, true, true>
    where
        OwnedResponse: for<'a> TryFrom<&'a crate::Response<'a>, Error = E>,
        E: Display,
    {
        GetBuilder {
            driver: self.driver,
            resources: self.resources,
            ke: self.ke,
            parameters: self.parameters,
            payload: self.payload,
            timeout: self.timeout,
            callback: Some(DynObject::new(AsyncCallback::new(
                async move |resp: &'_ crate::Response<'_>| {
                    let resp = OwnedResponse::try_from(resp);
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

impl<'this, 'res, Config, OwnedResponse, const CHANNEL: bool>
    GetBuilder<'this, 'res, Config, OwnedResponse, true, CHANNEL>
where
    Config: ZConfig,
{
    pub async fn finish(self) -> crate::ZResult<Responses<'this, OwnedResponse, CHANNEL>> {
        let timedout = Instant::now() + self.timeout.unwrap_or(Duration::from_secs(30));

        let rid = self.resources.next().await;

        if let Some(callback) = self.callback {
            let mut gets = self.resources.get_callbacks.lock().await;
            gets.drop_timedout();
            gets.insert(rid, self.ke, Some(timedout), callback)?;
        }

        let msg = Request {
            id: rid,
            wire_expr: WireExpr::from(self.ke),
            payload: RequestBody::Query(Query {
                consolidation: ConsolidationMode::None,
                parameters: self.parameters.unwrap_or_default(),
                body: self.payload.map(|p| Value {
                    payload: p,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };

        self.driver.send(msg).await?;

        Ok(Responses {
            timedout,
            receiver: self.receiver,
        })
    }
}

impl<'this, 'res, Config> super::Session<'this, 'res, Config>
where
    Config: ZConfig,
{
    pub fn get(&self, ke: &'static keyexpr) -> GetBuilder<'this, 'res, Config> {
        GetBuilder::new(self.driver, self.resources, ke)
    }
}
