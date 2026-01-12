use core::fmt::Display;

use dyn_utils::DynObject;
use embassy_sync::channel::{DynamicReceiver, DynamicSender};
use zenoh_proto::{fields::*, msgs::*, *};

use crate::api::{
    ZConfig,
    arg::QueryRef,
    callbacks::{AsyncCallback, DynCallback, SyncCallback, ZCallbacks},
    driver::Driver,
    resources::SessionResources,
};

pub struct Queryable<Config, OwnedQuery = (), const CHANNEL: bool = false>
where
    Config: ZConfig,
    OwnedQuery: 'static,
{
    driver: &'static Driver<'static, Config>,
    resources: &'static SessionResources<'static, Config>,

    id: u32,
    receiver: Option<DynamicReceiver<'static, OwnedQuery>>,
}

impl<Config, OwnedQuery, const CHANNEL: bool> Queryable<Config, OwnedQuery, CHANNEL>
where
    Config: ZConfig,
{
    pub async fn undeclare(self) -> crate::ZResult<()> {
        let msg = Declare {
            body: DeclareBody::UndeclareQueryable(UndeclareQueryable {
                id: self.id,
                ..Default::default()
            }),
            ..Default::default()
        };

        self.resources
            .queryable_callbacks
            .lock()
            .await
            .remove(self.id)?;

        self.driver.send(msg).await?;

        todo!("Also stop the channel if any")
    }
}

impl<Config, OwnedQuery> Queryable<Config, OwnedQuery, true>
where
    Config: ZConfig,
{
    pub fn try_recv(&self) -> Option<OwnedQuery> {
        self.receiver.as_ref().unwrap().try_receive().ok()
    }

    pub async fn recv(&self) -> Option<OwnedQuery> {
        Some(self.receiver.as_ref().unwrap().receive().await)
    }
}

pub struct QueryableBuilder<
    Config,
    OwnedQuery = (),
    const READY: bool = false,
    const CHANNEL: bool = false,
> where
    Config: ZConfig,
    OwnedQuery: 'static,
{
    driver: &'static Driver<'static, Config>,
    resources: &'static SessionResources<'static, Config>,

    ke: &'static keyexpr,

    callback: Option<
        DynCallback<
            'static,
            <Config::QueryableCallbacks<'static> as ZCallbacks<
                'static,
                QueryRef<'static, Config>,
            >>::Callback,
            <Config::QueryableCallbacks<'static> as ZCallbacks<
                'static,
                QueryRef<'static, Config>,
            >>::Future,
            QueryRef<'static, Config>,
        >,
    >,
    receiver: Option<DynamicReceiver<'static, OwnedQuery>>,
}

impl<Config> QueryableBuilder<Config, (), false, false>
where
    Config: ZConfig,
{
    pub(crate) fn new(
        driver: &'static Driver<'static, Config>,
        resources: &'static SessionResources<'static, Config>,
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
        callback: impl AsyncFnMut(&crate::Query<'_, 'static, Config>) + 'static,
    ) -> QueryableBuilder<Config, (), true, false> {
        QueryableBuilder {
            driver: self.driver,
            resources: self.resources,
            ke: self.ke,
            callback: Some(DynObject::new(AsyncCallback::new(callback))),
            receiver: None,
        }
    }

    pub fn callback_sync(
        self,
        callback: impl FnMut(&crate::Query<'_, 'static, Config>) + 'static,
    ) -> QueryableBuilder<Config, (), true, false> {
        QueryableBuilder {
            driver: self.driver,
            resources: self.resources,
            ke: self.ke,
            callback: Some(DynObject::new(SyncCallback::new(callback))),
            receiver: None,
        }
    }
}

impl<Config> QueryableBuilder<Config, (), false, false>
where
    Config: ZConfig,
{
    pub fn channel<OwnedQuery, E>(
        self,
        sender: DynamicSender<'static, OwnedQuery>,
        receiver: DynamicReceiver<'static, OwnedQuery>,
    ) -> QueryableBuilder<Config, OwnedQuery, true, true>
    where
        OwnedQuery: for<'any> TryFrom<
                (
                    &'any crate::Query<'any, 'static, Config>,
                    &'static Driver<'static, Config>,
                    &'static SessionResources<'static, Config>,
                ),
                Error = E,
            >,
        E: Display,
    {
        QueryableBuilder {
            driver: self.driver,
            resources: self.resources,
            ke: self.ke,
            callback: Some(DynObject::new(AsyncCallback::new(
                async move |resp: &'_ crate::Query<'_, 'static, Config>| {
                    let resp = OwnedQuery::try_from((resp, self.driver, self.resources));
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

impl<Config, OwnedQuery, const CHANNEL: bool> QueryableBuilder<Config, OwnedQuery, true, CHANNEL>
where
    Config: ZConfig,
{
    pub async fn finish(self) -> crate::ZResult<Queryable<Config, OwnedQuery, CHANNEL>> {
        let id = self.resources.next().await;

        if let Some(callback) = self.callback {
            let mut queryables = self.resources.queryable_callbacks.lock().await;
            queryables.drop_timedout();
            queryables.insert(id, self.ke, None, callback)?;
        }

        let msg = Declare {
            body: DeclareBody::DeclareQueryable(DeclareQueryable {
                id,
                wire_expr: WireExpr::from(self.ke),
                ..Default::default()
            }),
            ..Default::default()
        };

        self.driver.send(msg).await?;

        Ok(Queryable {
            id,
            driver: self.driver,
            resources: self.resources,
            receiver: self.receiver,
        })
    }
}

impl<Config> super::Session<'static, Config>
where
    Config: ZConfig,
{
    pub fn declare_queryable(&'static self, ke: &'static keyexpr) -> QueryableBuilder<Config> {
        QueryableBuilder::new(&self.driver, &self.resources, ke)
    }
}
