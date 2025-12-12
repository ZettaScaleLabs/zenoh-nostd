use crate::api::{
    HeaplessCallbacks, HeaplessChannels, HeaplessResponse, ResponseRef, SessionResources,
    ZCallbacks, ZChannel, ZChannels, ZConfig, driver::Driver,
};
use embassy_time::{Duration, Instant};
use zenoh_proto::{exts::*, fields::*, keyexpr, msgs::*};

pub type HeaplessGetCallbacks<
    const CAPACITY: usize,
    const CALLBACK_SIZE: usize = { size_of::<usize>() },
    const FUTURE_SIZE: usize = { 4 * size_of::<usize>() },
    const CALLBACK_ALIGN: usize = { size_of::<usize>() },
    const FUTURE_ALIGN: usize = { size_of::<usize>() },
> = HeaplessCallbacks<
    ResponseRef,
    (),
    CAPACITY,
    CALLBACK_SIZE,
    FUTURE_SIZE,
    CALLBACK_ALIGN,
    FUTURE_ALIGN,
>;

pub type HeaplessGetChannels<
    const MAX_KEYEXPR: usize,
    const MAX_PAYLOAD: usize,
    const QUEUED: usize,
    const CAPACITY: usize,
> = HeaplessChannels<HeaplessResponse<MAX_KEYEXPR, MAX_PAYLOAD>, QUEUED, CAPACITY>;

pub struct Get<'r, Config>
where
    Config: ZConfig,
{
    ke: &'static keyexpr,
    timedout: Instant,
    channel: Option<&'r <Config::GetChannels as ZChannels<ResponseRef>>::Channel>,
}

impl<'r, Config> Get<'r, Config>
where
    Config: ZConfig,
{
    pub fn keyexpr(&self) -> &'static keyexpr {
        self.ke
    }

    pub async fn recv(
        &self,
    ) -> core::result::Result<
        <<Config::GetChannels as ZChannels<ResponseRef>>::Channel as ZChannel<ResponseRef>>::Item,
        crate::SessionError,
    > {
        if let Some(ch) = &self.channel {
            Ok(
                match embassy_futures::select::select(
                    embassy_time::Timer::at(self.timedout),
                    ch.recv(),
                )
                .await
                {
                    embassy_futures::select::Either::First(_) => {
                        return Err(crate::SessionError::RequestTimedout);
                    }
                    embassy_futures::select::Either::Second(response) => response,
                },
            )
        } else {
            Err(crate::SessionError::ChannelClosed)
        }
    }
}

pub struct GetBuilder<'a, 'r, Config>
where
    Config: ZConfig,
{
    driver: &'a Driver<'r, Config>,
    resources: &'r SessionResources<Config>,

    ke: &'static keyexpr,
    parameters: Option<&'a str>,
    payload: Option<&'a [u8]>,
    timeout: Option<Duration>,

    callback: Option<<Config::GetCallbacks as ZCallbacks<ResponseRef, ()>>::Callback>,
}

impl<'a, 'r, Config> GetBuilder<'a, 'r, Config>
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
            parameters: None,
            payload: None,
            timeout: None,
            callback: None,
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

    pub fn callback(
        mut self,
        callback: <Config::GetCallbacks as ZCallbacks<ResponseRef, ()>>::Callback,
    ) -> Self {
        self.callback = Some(callback);
        self
    }

    pub async fn finish(self) -> crate::ZResult<Get<'r, Config>> {
        let timedout = Instant::now() + self.timeout.unwrap_or(Duration::from_secs(30));

        let rid = self.resources.next().await;

        let mut gets = self.resources.get_callbacks.lock().await;
        gets.drop_timedout();

        let channel = if let Some(callback) = self.callback {
            gets.insert(rid, self.ke, Some(timedout), callback)?;
            None
        } else {
            let channels = &self.resources.get_channels;
            let ch = channels.insert(rid, self.ke, Some(timedout)).await?;
            Some(ch)
        };

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

        Ok(Get {
            ke: self.ke,
            timedout,
            channel,
        })
    }
}

impl<'r, Config> super::Session<'r, Config>
where
    Config: ZConfig,
{
    pub fn get<'a>(&'a self, ke: &'static keyexpr) -> GetBuilder<'a, 'r, Config> {
        GetBuilder::new(self.driver, self.resources, ke)
    }
}
