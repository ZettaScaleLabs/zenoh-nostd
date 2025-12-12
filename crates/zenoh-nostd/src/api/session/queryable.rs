use crate::api::{
    HeaplessCallbacks, HeaplessChannels, HeaplessQuery, QueryRef, SessionResources, ZCallbacks,
    ZConfig, driver::Driver,
};
use zenoh_proto::{fields::*, keyexpr, msgs::*};

pub type HeaplessQueryableCallbacks<
    Config,
    const CAPACITY: usize,
    const CALLBACK_SIZE: usize = { size_of::<usize>() },
    const FUTURE_SIZE: usize = { 4 * size_of::<usize>() },
    const CALLBACK_ALIGN: usize = { size_of::<usize>() },
    const FUTURE_ALIGN: usize = { size_of::<usize>() },
> = HeaplessCallbacks<
    QueryRef<Config>,
    (),
    CAPACITY,
    CALLBACK_SIZE,
    FUTURE_SIZE,
    CALLBACK_ALIGN,
    FUTURE_ALIGN,
>;

pub type HeaplessQueryableChannels<
    Config,
    const MAX_KEYEXPR: usize,
    const MAX_PARAMETERS: usize,
    const MAX_PAYLOAD: usize,
    const QUEUED: usize,
    const CAPACITY: usize,
> = HeaplessChannels<
    HeaplessQuery<MAX_KEYEXPR, MAX_PARAMETERS, MAX_PAYLOAD, Config>,
    QUEUED,
    CAPACITY,
>;

pub struct Queryable<'a, 'r, Config>
where
    Config: ZConfig,
{
    driver: &'a Driver<'r, Config>,
    resources: &'r SessionResources<Config>,

    ke: &'static keyexpr,
    id: u32,
}

impl<'r, Config> Queryable<'_, 'r, Config>
where
    Config: ZConfig,
{
    pub fn keyexpr(&self) -> &'static keyexpr {
        self.ke
    }

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

        Ok(())
    }
}

pub struct QueryableBuilder<'a, 'r, Config>
where
    Config: ZConfig,
{
    driver: &'a Driver<'r, Config>,
    resources: &'r SessionResources<Config>,

    ke: &'static keyexpr,

    callback: Option<<Config::QueryableCallbacks as ZCallbacks<QueryRef<Config>, ()>>::Callback>,
}

impl<'a, 'r, Config> QueryableBuilder<'a, 'r, Config>
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
        callback: <Config::QueryableCallbacks as ZCallbacks<QueryRef<Config>, ()>>::Callback,
    ) -> Self {
        self.callback = Some(callback);
        self
    }

    pub async fn finish(self) -> crate::ZResult<Queryable<'a, 'r, Config>> {
        let id = self.resources.next().await;

        let mut queryables = self.resources.queryable_callbacks.lock().await;
        if let Some(callback) = self.callback {
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
            driver: self.driver,
            resources: self.resources,
            ke: self.ke,
            id,
        })
    }
}

impl<'r, Config> super::Session<'r, Config>
where
    Config: ZConfig,
{
    pub fn declare_queryable<'a>(
        &'a self,
        ke: &'static keyexpr,
    ) -> QueryableBuilder<'a, 'r, Config> {
        QueryableBuilder::new(self.driver, self.resources, ke)
    }
}
