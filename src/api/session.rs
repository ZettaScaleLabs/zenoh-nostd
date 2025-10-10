use crate::{
    api::{
        ZConfig,
        driver::SessionDriver,
        subscriber::{Subscriber, ZCallback},
    },
    io::{
        link::Link,
        transport::{TransportMineConfig, establishment::open::open_link},
    },
    keyexpr::borrowed::keyexpr,
    platform::Platform,
    protocol::{
        core::{
            Reliability, ZenohIdProto, encoding::Encoding, endpoint::EndPoint, wire_expr::WireExpr,
        },
        network::{
            self, NetworkBody, NetworkMessage,
            declare::{Declare, DeclareBody, subscriber::DeclareSubscriber},
            push::Push,
        },
        zenoh::{PushBody, put::Put},
    },
    result::ZResult,
};

pub struct Session<T: Platform + 'static> {
    driver: Option<&'static SessionDriver<T>>,

    next_id: u32,
}

impl<T: Platform + 'static> Session<T> {
    pub async fn new(config: ZConfig<T>, endpoint: EndPoint) -> ZResult<(Self, SessionDriver<T>)> {
        let transport = TransportMineConfig {
            mine_zid: ZenohIdProto::default(),
            mine_lease: core::time::Duration::from_secs(20),
            keep_alive: 4,
            open_timeout: core::time::Duration::from_secs(5),
        };

        let link = Link::new(&config.platform, endpoint).await?;
        let (transport, tconfig) =
            open_link(link, transport, config.tx_zbuf, config.rx_zbuf).await?;

        let (tx, rx) = transport.split();

        Ok((
            Self {
                driver: None,
                next_id: 0,
            },
            SessionDriver::new(
                tconfig,
                (config.tx_zbuf, tx),
                (config.rx_zbuf, rx),
                config.callbacks,
            ),
        ))
    }

    pub fn set_driver(&mut self, driver: &'static SessionDriver<T>) {
        self.driver = Some(driver);
    }

    pub async fn put(&mut self, ke: &'static keyexpr, bytes: &[u8]) -> ZResult<()> {
        let msg = NetworkMessage {
            reliability: Reliability::DEFAULT,
            body: NetworkBody::Push(Push {
                wire_expr: WireExpr::from(ke),
                ext_qos: network::ext::QoSType::DEFAULT,
                ext_tstamp: None,
                ext_nodeid: network::push::ext::NodeIdType::DEFAULT,
                payload: PushBody::Put(Put {
                    timestamp: None,
                    encoding: Encoding::empty(),
                    ext_sinfo: None,
                    ext_attachment: None,
                    payload: bytes,
                }),
            }),
        };

        self.driver.as_ref().unwrap().send(msg).await?;

        Ok(())
    }

    pub async fn declare_subscriber(
        &mut self,
        ke: &'static keyexpr,
        callback: ZCallback,
    ) -> ZResult<Subscriber> {
        let ke = WireExpr::from(ke);

        let id = self.next_id;
        self.next_id += 1;

        let msg = NetworkMessage {
            reliability: Reliability::Reliable,
            body: NetworkBody::Declare(Declare {
                interest_id: None,
                ext_qos: network::declare::ext::QoSType::DECLARE,
                ext_tstamp: None,
                ext_nodeid: network::declare::ext::NodeIdType::DEFAULT,
                body: DeclareBody::DeclareSubscriber(DeclareSubscriber {
                    id,
                    wire_expr: ke.clone(),
                }),
            }),
        };

        self.driver.as_ref().unwrap().send(msg).await?;

        self.driver
            .as_ref()
            .unwrap()
            .register_callback(ke, callback)
            .await?;

        Ok(Subscriber::Sync)
    }
}
