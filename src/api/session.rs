use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use heapless::index_map::FnvIndexMap;

use crate::{
    io::{
        link::Link,
        transport::{
            SingleLinkTransport, SingleLinkTransportConfig, SingleLinkTransportMineConfig,
        },
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
        transport::{
            self, TransportBody, TransportMessage, TransportSn,
            frame::{Frame, FrameHeader},
            init::{InitAck, InitSyn},
            keepalive::KeepAlive,
            open::{OpenAck, OpenSyn},
        },
        zenoh::{PushBody, put::Put},
    },
    result::ZResult,
    zbuf::{ZBuf, ZBufMut, ZBufReader},
};

pub struct SessionDriver<T: Platform> {
    config: SingleLinkTransportConfig,
    link: Mutex<CriticalSectionRawMutex, SingleLinkTransport<T>>,
}

impl<T: Platform> SessionDriver<T> {
    pub async fn run(&self) {
        let timeout =
            self.config.mine_config.mine_lease / self.config.mine_config.keep_alive as u32;

        let mut tx_zbuf = [0u8; 32];

        loop {
            embassy_time::Timer::after(timeout.try_into().unwrap()).await;

            let mut link = self.link.lock().await;
            link.send(
                tx_zbuf.as_mut_slice(),
                &TransportMessage {
                    body: TransportBody::KeepAlive(KeepAlive),
                },
            )
            .await
            .unwrap();

            drop(link);
        }
    }
}

pub struct Session<T: Platform + 'static> {
    _platform: T,
    current_sn: TransportSn,

    driver: Option<&'static SessionDriver<T>>,

    next_id: u32,
    mapping: FnvIndexMap<WireExpr<'static>, u32, 16>,
}

impl<T: Platform + 'static> Session<T> {
    pub async fn new(platform: T, endpoint: EndPoint) -> ZResult<(Self, SessionDriver<T>)> {
        let config = SingleLinkTransportMineConfig {
            mine_zid: ZenohIdProto::default(),
            mine_lease: core::time::Duration::from_secs(20),
            keep_alive: 4,
            open_timeout: core::time::Duration::from_secs(5),
        };

        let link = Link::new(&platform, endpoint).await?;
        let (link, config) = SingleLinkTransport::open::<256, 256>(link, config).await?;

        Ok((
            Self {
                _platform: platform,
                current_sn: config.negociated_config.mine_sn + 1,
                driver: None,
                next_id: 0,
                mapping: FnvIndexMap::new(),
            },
            SessionDriver {
                config,
                link: Mutex::new(link),
            },
        ))
    }

    pub fn set_driver(&mut self, driver: &'static SessionDriver<T>) {
        self.driver = Some(driver);
    }

    pub async fn put(
        &mut self,
        tx_zbuf: ZBufMut<'_>,
        ke: &'static keyexpr,
        bytes: &[u8],
    ) -> ZResult<()> {
        let msg = NetworkMessage {
            reliability: Reliability::DEFAULT,
            body: NetworkBody::Push(Push {
                wire_expr: WireExpr::from(ke),
                ext_qos: network::ext::QoSType::PUSH,
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

        let frame = Frame {
            reliability: Reliability::DEFAULT,
            sn: self.current_sn,
            ext_qos: transport::frame::ext::QoSType::DEFAULT,
            payload: &[msg],
        };

        let tmsg = TransportMessage {
            body: TransportBody::Frame(frame),
        };

        let mut link = self.driver.as_ref().unwrap().link.lock().await;
        link.send(tx_zbuf, &tmsg).await?;

        self.current_sn = self.current_sn.wrapping_add(1);

        Ok(())
    }

    pub async fn declare_subscription(
        &mut self,
        tx_zbuf: ZBufMut<'_>,
        ke: &'static keyexpr,
    ) -> ZResult<u32> {
        let ke = WireExpr::from(ke);

        let id = self.next_id;
        self.next_id += 1;

        self.mapping.insert(ke.clone(), id).unwrap();

        let msg = NetworkMessage {
            reliability: Reliability::Reliable,
            body: NetworkBody::Declare(Declare {
                interest_id: None,
                ext_qos: network::declare::ext::QoSType::DECLARE,
                ext_tstamp: None,
                ext_nodeid: network::declare::ext::NodeIdType::DEFAULT,
                body: DeclareBody::DeclareSubscriber(DeclareSubscriber { id, wire_expr: ke }),
            }),
        };

        let frame = Frame {
            reliability: Reliability::Reliable,
            sn: self.current_sn,
            ext_qos: transport::frame::ext::QoSType::DEFAULT,
            payload: &[msg],
        };

        let tmsg = TransportMessage {
            body: TransportBody::Frame(frame),
        };

        let mut link = self.driver.as_ref().unwrap().link.lock().await;
        link.send(tx_zbuf, &tmsg).await?;

        self.current_sn = self.current_sn.wrapping_add(1);

        Ok(id)
    }
}

impl<T: Platform> Session<T> {
    pub async fn read<'a>(
        &self,
        rx_buffer: ZBufMut<'a>,
        mut on_sample: impl AsyncFnMut(u32, ZBuf<'a>),
    ) -> ZResult<()> {
        let mut link = self.driver.as_ref().unwrap().link.lock().await;
        let mut reader: ZBufReader<'a> = link.recv(rx_buffer).await?;

        TransportMessage::decode_batch_async(
            &mut reader,
            None::<fn(InitSyn) -> ZResult<()>>,
            None::<fn(InitAck) -> ZResult<()>>,
            None::<fn(OpenSyn) -> ZResult<()>>,
            None::<fn(OpenAck) -> ZResult<()>>,
            None::<fn() -> ZResult<()>>,
            Some(async |_: &FrameHeader, msg: NetworkMessage<'a>| {
                if let NetworkBody::Push(push) = msg.body {
                    // TODO: confronting the wire_expr with the mapping is not sufficient. If we subcribed to foo/* we may receive foo/bar...
                    let id = self.mapping.get(&push.wire_expr).copied().unwrap();

                    match push.payload {
                        PushBody::Put(put) => {
                            let zbuf: ZBuf<'a> = put.payload;
                            on_sample(id, zbuf).await;
                        }
                    }
                }

                Ok(())
            }),
        )
        .await?;

        Ok(())
    }

    pub async fn try_read<'a>(
        &self,
        rx_buffer: ZBufMut<'a>,
        mut callback: impl AsyncFnMut(u32, ZBuf<'a>),
        timeout: core::time::Duration,
    ) -> ZResult<()> {
        embassy_time::with_timeout(
            timeout.try_into().unwrap(),
            self.read(rx_buffer, &mut callback),
        )
        .await
        .map_err(|_| crate::result::ZError::TimedOut)?
    }
}
