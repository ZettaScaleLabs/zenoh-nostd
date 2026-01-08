use dyn_utils::DynObject;
use embassy_time::{Duration, Instant};
use zenoh_proto::{exts::*, fields::*, msgs::*, *};

use crate::{
    Resources,
    api::{
        ZConfig,
        arg::ResponseRef,
        callbacks::{DynCallback, SyncCallback, ZCallbacks},
        driver::Driver,
        resources::SessionResources,
    },
};

pub struct GetBuilder<'a, Config>
where
    Config: ZConfig,
{
    driver: &'a Driver<'a, Config>,
    resources: &'a SessionResources<'a, Config>,

    ke: &'static keyexpr,
    parameters: Option<&'a str>,
    payload: Option<&'a [u8]>,
    timeout: Option<Duration>,
    // callback: DynCallback<
    //     'a,
    //     <Config::GetCallbacks<'a> as ZCallbacks<'a, ResponseRef>>::Callback,
    //     <Config::GetCallbacks<'a> as ZCallbacks<'a, ResponseRef>>::Future,
    //     ResponseRef,
    // >,
}

impl<'a, Config> GetBuilder<'a, Config>
where
    Config: ZConfig,
{
    pub(crate) fn new(
        driver: &'a Driver<'a, Config>,
        resources: &'a SessionResources<'a, Config>,
        ke: &'static keyexpr,
    ) -> Self {
        Self {
            driver,
            resources,
            ke,
            parameters: None,
            payload: None,
            timeout: None,
            // callback: DynObject::new(SyncCallback::new(|_: &Response<'_>| {})),
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

    // pub fn callback(
    //     mut self,
    //     callback: DynCallback<
    //         'a,
    //         <Config::GetCallbacks<'a> as ZCallbacks<'a, ResponseRef>>::Callback,
    //         <Config::GetCallbacks<'a> as ZCallbacks<'a, ResponseRef>>::Future,
    //         ResponseRef,
    //     >,
    // ) -> Self {
    //     self.callback = callback;
    //     self
    // }

    pub async fn finish(self) -> crate::ZResult<()> {
        let timedout = Instant::now() + self.timeout.unwrap_or(Duration::from_secs(30));

        let rid = self.resources.next().await;

        let mut gets = self.resources.get_callbacks.lock().await;
        gets.drop_timedout();

        // gets.insert(rid, self.ke, Some(timedout), self.callback)?;

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

        Ok(())
    }
}

pub struct TestTest<'this, 'res, Config>
where
    Config: ZConfig,
{
    driver: &'this Driver<'this, Config>,
    resources: &'this SessionResources<'res, Config>,

    ke: &'static keyexpr,
    parameters: Option<&'this str>,
    payload: Option<&'this [u8]>,
    timeout: Option<Duration>,
}

impl<'this, 'res, Config> TestTest<'this, 'res, Config>
where
    Config: ZConfig,
{
    pub async fn finish(self) -> crate::ZResult<()> {
        let timedout = Instant::now() + self.timeout.unwrap_or(Duration::from_secs(30));

        let rid = self.resources.next().await;

        let mut gets = self.resources.get_callbacks.lock().await;
        gets.drop_timedout();

        // gets.insert(rid, self.ke, Some(timedout), self.callback)?;

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

        Ok(())
    }
}

impl<'this, 'res, Config> super::Session<'this, 'res, Config>
where
    Config: ZConfig,
{
    pub fn testtest(&self, ke: &'static keyexpr) -> TestTest<'this, 'res, Config> {
        TestTest {
            driver: self.driver,
            resources: self.resources,
            ke,
            parameters: None,
            payload: None,
            timeout: None,
        }
    }

    pub fn get(&self, ke: &'static keyexpr) -> GetBuilder<'this, Config> {
        let driver: &'this Driver<'this, Config> = self.driver;
        let res: &'this SessionResources<'res, Config> = self.resources;
        // GetBuilder::new(self.driver, self.resources, ke)

        todo!()
    }
}
