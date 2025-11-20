use embassy_sync::channel::DynamicReceiver;
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
    ZPublisher,
    api::{
        ZConfig, callback::ZCallback, driver::SessionDriver, sample::ZOwnedSample,
        subscriber::ZSubscriber,
    },
    io::{
        link::Link,
        transport::{Transport, TransportMineConfig},
    },
    platform::Platform,
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
            ),
        ))
    }

    pub fn set_driver(&mut self, driver: &'static SessionDriver<T>) {
        self.driver = Some(driver);
    }

    pub async fn put(&mut self, ke: &'static keyexpr, bytes: &[u8]) -> ZResult<()> {
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
            ZCallback,
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
}
