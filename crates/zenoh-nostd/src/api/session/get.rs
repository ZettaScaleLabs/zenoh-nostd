use core::time::Duration;

use dyn_utils::{DynObject, storage::RawOrBox};
use embassy_futures::select::{Either, select};
use embassy_sync::channel::{DynamicReceiver, DynamicSender};
use embassy_time::{Instant, Timer};
use zenoh_proto::{
    SessionError,
    exts::{QoS, Value},
    fields::{ConsolidationMode, Reliability, WireExpr},
    keyexpr,
    msgs::{NetworkBody, NetworkMessage, Query, Request, RequestBody},
};

use crate::{
    api::{
        arg::GetResponseRef,
        callbacks::{AsyncCallback, DynCallback, FixedCapacityCallbacks, SyncCallback, ZCallbacks},
        session::Session,
    },
    config::ZSessionConfig,
    io::transport::ZTransportLinkTx,
    session::GetResponse,
};

pub type FixedCapacityGetCallbacks<
    'a,
    const CAPACITY: usize,
    Callback = RawOrBox<16>,
    Future = RawOrBox<128>,
> = FixedCapacityCallbacks<'a, GetResponseRef, CAPACITY, Callback, Future>;

pub struct GetResponses<'res, OwnedResponse = (), const CHANNEL: bool = false> {
    ke: &'static keyexpr,
    timedout: Instant,
    receiver: Option<DynamicReceiver<'res, OwnedResponse>>,
}

impl<'res, OwnedResponse, const CHANNEL: bool> GetResponses<'res, OwnedResponse, CHANNEL> {
    pub fn keyexpr(&self) -> &keyexpr {
        &self.ke
    }
}

impl<'res, OwnedResponse> GetResponses<'res, OwnedResponse, true> {
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

type CallbackStorage<'res, Config> =
    <<Config as ZSessionConfig>::GetCallbacks<'res> as ZCallbacks<'res, GetResponseRef>>::Callback;

type FutureStorage<'res, Config> =
    <<Config as ZSessionConfig>::GetCallbacks<'res> as ZCallbacks<'res, GetResponseRef>>::Future;

pub struct GetBuilder<
    'parameters,
    'session,
    'ext,
    'res,
    Config,
    OwnedResponse = (),
    const READY: bool = false,
    const CHANNEL: bool = false,
> where
    Config: ZSessionConfig,
{
    pub(crate) session: &'session Session<'ext, 'res, Config>,
    pub(crate) ke: &'static keyexpr,
    pub(crate) parameters: Option<&'parameters str>,
    pub(crate) payload: Option<&'parameters [u8]>,
    pub(crate) timeout: Option<Duration>,
    pub(crate) callback: Option<
        DynCallback<
            'res,
            CallbackStorage<'res, Config>,
            FutureStorage<'res, Config>,
            GetResponseRef,
        >,
    >,
    pub(crate) receiver: Option<DynamicReceiver<'res, OwnedResponse>>,
}

impl<'parameters, 'session, 'ext, 'res, Config>
    GetBuilder<'parameters, 'session, 'ext, 'res, Config, (), false, false>
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
            callback: None,
            receiver: None,
        }
    }

    pub fn callback(
        self,
        callback: impl AsyncFnMut(&GetResponse<'_>) + 'res,
    ) -> GetBuilder<'parameters, 'session, 'ext, 'res, Config, (), true> {
        GetBuilder {
            session: self.session,
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
        callback: impl FnMut(&GetResponse<'_>) + 'res,
    ) -> GetBuilder<'parameters, 'session, 'ext, 'res, Config, (), true> {
        GetBuilder {
            session: self.session,
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
    ) -> GetBuilder<'parameters, 'session, 'ext, 'res, Config, OwnedResponse, true, true>
    where
        OwnedResponse: for<'any> TryFrom<&'any GetResponse<'any>, Error = E>,
    {
        GetBuilder {
            session: self.session,
            ke: self.ke,
            parameters: self.parameters,
            payload: self.payload,
            timeout: self.timeout,
            callback: Some(DynObject::new(AsyncCallback::new(
                async move |resp: &'_ GetResponse<'_>| {
                    if let Ok(resp) = OwnedResponse::try_from(resp) {
                        sender.send(resp).await;
                    } else {
                        zenoh_proto::error!(
                            "{}: Couldn't convert to a transferable response",
                            zenoh_proto::zctx!()
                        )
                    }
                },
            ))),
            receiver: Some(receiver),
        }
    }
}

impl<
    'parameters,
    'session,
    'ext,
    'res,
    Config,
    OwnedResponse,
    const READY: bool,
    const CHANNEL: bool,
> GetBuilder<'parameters, 'session, 'ext, 'res, Config, OwnedResponse, READY, CHANNEL>
where
    Config: ZSessionConfig,
{
    pub fn keyexpr(mut self, ke: &'static keyexpr) -> Self {
        self.ke = ke;
        self
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
}

impl<'parameters, 'session, 'ext, 'res, Config, OwnedResponse, const CHANNEL: bool>
    GetBuilder<'parameters, 'session, 'ext, 'res, Config, OwnedResponse, true, CHANNEL>
where
    Config: ZSessionConfig,
{
    pub async fn finish(
        self,
    ) -> core::result::Result<GetResponses<'res, OwnedResponse, CHANNEL>, SessionError> {
        let timedout = Instant::now()
            + self
                .timeout
                .unwrap_or(Duration::from_secs(30))
                .try_into()
                .unwrap();

        let mut state = self.session.state().await;
        let rid = state.next();

        if let Some(callback) = self.callback {
            state.get_callbacks.drop_timedout();
            state
                .get_callbacks
                .insert(rid, self.ke, Some(timedout), callback)?;
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

        self.session
            .driver
            .tx()
            .await
            .send(core::iter::once(NetworkMessage {
                reliability: Reliability::default(),
                qos: QoS::default(),
                body: NetworkBody::Request(msg),
            }))
            .await?;

        Ok(GetResponses {
            ke: self.ke,
            timedout,
            receiver: self.receiver,
        })
    }
}

impl<'ext, 'res, Config> Session<'ext, 'res, Config>
where
    Config: ZSessionConfig,
{
    pub fn get<'parameters>(
        &self,
        ke: &'static keyexpr,
    ) -> GetBuilder<'parameters, '_, 'ext, 'res, Config> {
        GetBuilder::new(self, ke)
    }
}
