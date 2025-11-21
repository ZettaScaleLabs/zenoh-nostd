use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, channel::DynamicReceiver, mutex::Mutex,
};
use zenoh_proto::{
    Encoding, EndPoint, WireExpr, ZResult, ZenohIdProto, keyexpr,
    network::{
        NetworkBody, NodeId, QoS,
        declare::{Declare, DeclareBody, DeclareSubscriber},
        push::Push,
    },
    zenoh::{PushBody, put::Put},
};

use crate::{
    ZPublisher, ZReply,
    api::{ZConfig, driver::SessionDriver, sample::ZOwnedSample, subscriber::ZSubscriber},
    io::{
        link::Link,
        transport::{Transport, TransportMineConfig},
    },
    platform::Platform,
    session::get::GetBuilder,
    subscriber::callback::ZSubscriberCallback,
};

pub mod get;

static NEXT_ID: Mutex<CriticalSectionRawMutex, u32> = Mutex::new(0);

pub struct Session<T: Platform + 'static> {
    driver: Option<&'static SessionDriver<T>>,
}

impl<T: Platform + 'static> Session<T> {
    pub async fn new<S1>(
        config: ZConfig<T, S1>,
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
            Self { driver: None },
            SessionDriver::new(
                tconfig,
                (config.tx_zbuf, tx),
                (config.rx_zbuf, rx),
                config.subscribers,
                config.replies,
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
        &self,
        ke: &'static keyexpr,
        config: (
            ZSubscriberCallback,
            Option<DynamicReceiver<'static, ZOwnedSample<KE, PL>>>,
        ),
    ) -> ZResult<ZSubscriber<KE, PL>> {
        let wke = WireExpr::from(ke);

        let mut id = NEXT_ID.lock().await;
        *id += 1;
        let id = *id;

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

    pub fn get(&self, ke: &'static keyexpr, callback: fn(&ZReply<'_>)) -> GetBuilder<'_, T> {
        GetBuilder::new(self, ke, callback)
    }

    // pub fn declare_querier(&self, ke: &'static keyexpr) -> ZQuerier<T, S> {
    //     ZQuerier::new(
    //         self.spawner,
    //         self.timeout_queries,
    //         ke,
    //         self.driver.as_ref().unwrap(),
    //         &NEXT_ID,
    //     )
    // }
}
