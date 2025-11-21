use embassy_sync::channel::DynamicReceiver;
use zenoh_proto::{
    Encoding, EndPoint, WireExpr, ZResult, ZenohIdProto, keyexpr,
    network::{
        NetworkBody, NodeId, QoS, QueryTarget,
        declare::{Declare, DeclareBody, DeclareSubscriber},
        push::Push,
        request::Request,
    },
    zenoh::{ConsolidationMode, PushBody, RequestBody, Value, put::Put, query::Query},
};

use crate::{
    ZOwnedReply, ZPublisher, ZQuery, ZQueryCallback,
    api::{ZConfig, driver::SessionDriver, sample::ZOwnedSample, subscriber::ZSubscriber},
    io::{
        link::Link,
        transport::{Transport, TransportMineConfig},
    },
    platform::Platform,
    subscriber::callback::ZSubscriberCallback,
};

pub struct Session<T: Platform + 'static> {
    driver: Option<&'static SessionDriver<T>>,

    next_id: u32,
}

impl<T: Platform + 'static> Session<T> {
    pub async fn new<S>(
        config: ZConfig<T, S>,
        endpoint: EndPoint,
    ) -> ZResult<(Self, SessionDriver<T>)> {
        let transport = TransportMineConfig {
            mine_zid: ZenohIdProto::default(),
            mine_lease: core::time::Duration::from_secs(20),
            keep_alive: 4,
            open_timeout: core::time::Duration::from_secs(5),
        };

        let link = Link::new(&config.platform, endpoint).await?;
        let (transport, tconfig) =
            Transport::open(link, transport, config.tx_zbuf, config.rx_zbuf).await?;

        let (tx, rx) = config.transport.init(transport).split();

        Ok((
            Self {
                driver: None,
                next_id: 0,
            },
            SessionDriver::new(
                tconfig,
                (config.tx_zbuf, tx),
                (config.rx_zbuf, rx),
                config.subscribers,
                config.queries,
            ),
        ))
    }

    pub fn set_driver(&mut self, driver: &'static SessionDriver<T>) {
        self.driver = Some(driver);
    }

    pub async fn put(&self, ke: &'static keyexpr, bytes: &[u8]) -> ZResult<()> {
        let msg = NetworkBody::Push(Push {
            wire_expr: WireExpr::from(ke),
            qos: QoS::DEFAULT,
            timestamp: None,
            nodeid: NodeId::DEFAULT,
            payload: PushBody::Put(Put {
                timestamp: None,
                encoding: Encoding::empty(),
                sinfo: None,
                attachment: None,
                payload: bytes,
            }),
        });

        self.driver.as_ref().unwrap().send(msg).await?;

        Ok(())
    }

    pub async fn declare_subscriber<const KE: usize, const PL: usize>(
        &mut self,
        ke: &'static keyexpr,
        config: (
            ZSubscriberCallback,
            Option<DynamicReceiver<'static, ZOwnedSample<KE, PL>>>,
        ),
    ) -> ZResult<ZSubscriber<KE, PL>> {
        let wke = WireExpr::from(ke);

        let id = self.next_id;
        self.next_id += 1;

        let msg = NetworkBody::Declare(Declare {
            id: None,
            qos: QoS::DECLARE,
            timestamp: None,
            nodeid: NodeId::DEFAULT,
            body: DeclareBody::DeclareSubscriber(DeclareSubscriber { id, wire_expr: wke }),
        });

        self.driver.as_ref().unwrap().send(msg).await?;

        let is_async = config.0.is_async();

        self.driver
            .as_ref()
            .unwrap()
            .register_subscriber_callback(id, ke, config.0)
            .await?;

        if is_async {
            Ok(ZSubscriber::new_async(id, ke, config.1.unwrap()))
        } else {
            Ok(ZSubscriber::new_sync(id, ke))
        }
    }

    pub fn declare_publisher(&self, ke: &'static keyexpr) -> ZPublisher<T> {
        ZPublisher::new(ke, self.driver.as_ref().unwrap())
    }

    pub async fn get<const KE: usize, const PL: usize>(
        &mut self,
        ke: &'static keyexpr,
        parameters: Option<&str>,
        payload: Option<&[u8]>,
        config: (
            ZQueryCallback,
            Option<DynamicReceiver<'static, ZOwnedReply<KE, PL>>>,
        ),
    ) -> ZResult<ZQuery<KE, PL>> {
        let wke = WireExpr::from(ke);

        let id = self.next_id;
        self.next_id += 1;

        let is_async = config.0.is_async();

        self.driver
            .as_ref()
            .unwrap()
            .register_query_callback(id, ke, config.0)
            .await?;

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

        self.driver.as_ref().unwrap().send(msg).await?;

        if is_async {
            Ok(ZQuery::new_async(id, ke, config.1.unwrap()))
        } else {
            Ok(ZQuery::new_sync(id, ke))
        }
    }
}
