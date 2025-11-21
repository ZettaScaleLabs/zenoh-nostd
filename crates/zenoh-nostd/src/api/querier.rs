use embassy_executor::{SpawnToken, Spawner};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, channel::DynamicReceiver, mutex::Mutex,
};
use embassy_time::Duration;
use zenoh_proto::{
    Encoding, WireExpr, ZError, ZResult, keyexpr,
    network::{NetworkBody, NodeId, QoS, QueryTarget, request::Request},
    zenoh::{ConsolidationMode, RequestBody, Value, query::Query},
};

use crate::{SessionDriver, ZOwnedReply, ZReplies, ZRepliesCallback, platform::Platform};

pub struct ZQuerier<T: Platform + 'static, S> {
    spawner: Spawner,
    timeout_queries: fn(
        driver: &'static SessionDriver<T>,
        id: u32,
        timeout: embassy_time::Duration,
    ) -> SpawnToken<S>,
    ke: &'static keyexpr,
    driver: &'static SessionDriver<T>,

    next_id: &'static Mutex<CriticalSectionRawMutex, u32>,
}

impl<T: Platform + 'static, S> ZQuerier<T, S> {
    pub(crate) fn new(
        spawner: Spawner,
        timeout_queries: fn(
            driver: &'static SessionDriver<T>,
            id: u32,
            timeout: embassy_time::Duration,
        ) -> SpawnToken<S>,
        ke: &'static keyexpr,
        driver: &'static SessionDriver<T>,
        next_id: &'static Mutex<CriticalSectionRawMutex, u32>,
    ) -> Self {
        ZQuerier {
            spawner,
            timeout_queries,
            ke,
            driver,
            next_id,
        }
    }

    pub fn keyexpr(&self) -> &'static keyexpr {
        self.ke
    }

    pub async fn get<const KE: usize, const PL: usize>(
        &self,
        timeout: Option<Duration>,
        parameters: Option<&str>,
        payload: Option<&[u8]>,
        config: (
            ZRepliesCallback,
            Option<DynamicReceiver<'static, ZOwnedReply<KE, PL>>>,
        ),
    ) -> ZResult<ZReplies<KE, PL>> {
        let wke = WireExpr::from(self.ke);

        let mut id = self.next_id.lock().await;
        *id += 1;
        let id = *id;

        let is_async = config.0.is_async();

        self.driver
            .register_query_callback(id, self.ke, config.0)
            .await?;

        let timeout = timeout.unwrap_or(Duration::from_secs(5));

        let token = (self.timeout_queries)(self.driver, id, timeout);

        self.spawner
            .spawn((self.timeout_queries)(self.driver, id, timeout))
            .map_err(|_| ZError::CouldNotSpawnTask)?;

        let msg = NetworkBody::Request(Request {
            id,
            wire_expr: wke,
            payload: RequestBody::Query(Query {
                consolidation: ConsolidationMode::None,
                parameters: parameters.unwrap_or_default(),
                body: payload.map(|p| Value {
                    encoding: Encoding::empty(),
                    payload: p,
                }),
                attachment: None,
                sinfo: None,
            }),
            qos: QoS::DEFAULT,
            timestamp: None,
            nodeid: NodeId::DEFAULT,
            budget: None,
            timeout: None,
            target: QueryTarget::DEFAULT,
        });

        self.driver.send(msg).await?;

        if is_async {
            Ok(ZReplies::new_async(id, self.ke, timeout, config.1.unwrap()))
        } else {
            Ok(ZReplies::new_sync(id, self.ke, timeout))
        }
    }
}
