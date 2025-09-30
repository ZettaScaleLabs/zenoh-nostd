use embassy_futures::select::select;

use embassy_sync::{
    blocking_mutex::raw::NoopRawMutex,
    channel::{Channel, Receiver},
};
use embassy_time::Timer;
use heapless::{index_map::FnvIndexMap, Vec};
use static_cell::StaticCell;
use zenoh_buffers::{
    reader::HasReader,
    writer::HasWriter,
    zslice::{ArcBytes128, ArcBytes256, ArcBytes64, ZSlice, ZSliceLen},
    zunsafe_arc_pool_init,
};
use zenoh_codec::{RCodec, WCodec, Zenoh080};
use zenoh_platform::Platform;
use zenoh_protocol::{
    core::{key_expr::keyexpr, Encoding, EndPoint, WireExpr, ZenohIdProto},
    network::{
        declare::{Declare, DeclareBody, DeclareSubscriber},
        push::Push,
        NetworkBody, NetworkMessage,
    },
    transport::{Frame, KeepAlive, TransportMessage},
    zenoh::{PushBody, Put},
};
use zenoh_result::{bail, zctx, zerr, WithContext, ZResult, ZE};
use zenoh_transport::{
    unicast::{
        link::TransportLinkUnicast,
        open::{RecvOpenAckOut, SendOpenSynOut},
    },
    TransportManager,
};

pub struct SessionRunner<'a, T: Platform> {
    link: TransportLinkUnicast<T, 32, 32>,

    send_open_syn: SendOpenSynOut,
    recv_open_ack: RecvOpenAckOut,
    tm: TransportManager<'a, T>,

    session_to_transport: &'static Channel<NoopRawMutex, NetworkMessage, 8>,
    transport_to_session: &'static Channel<NoopRawMutex, TransportMessage, 8>,
}

impl<T: Platform> SessionRunner<'_, T> {
    pub async fn run(&mut self) {
        let mut sn = self.send_open_syn.mine_initial_sn + 1;

        let keep_alive_timeout = self.tm.unicast.lease / (self.tm.unicast.keep_alive as u32);
        let other_lease = self.recv_open_ack.other_lease;

        let mut last_read_time = embassy_time::Instant::now();
        let mut last_write_time = embassy_time::Instant::now();

        'main: loop {
            let read_lease = Timer::at(last_read_time + other_lease.try_into().unwrap());
            let write_lease = Timer::at(last_write_time + keep_alive_timeout.try_into().unwrap());

            match select(
                select(read_lease, self.link.recv::<256>()),
                select(write_lease, self.session_to_transport.receive()),
            )
            .await
            {
                embassy_futures::select::Either::First(read_task) => {
                    last_read_time = embassy_time::Instant::now();

                    match read_task {
                        embassy_futures::select::Either::First(_) => {
                            break 'main;
                        }
                        embassy_futures::select::Either::Second(msg) => match msg {
                            Ok(msg) => {
                                self.transport_to_session.send(msg).await;
                            }
                            Err(_) => {
                                break 'main;
                            }
                        },
                    }
                }
                embassy_futures::select::Either::Second(write_task) => {
                    last_write_time = embassy_time::Instant::now();

                    match write_task {
                        embassy_futures::select::Either::First(_) => {
                            if let Err(_) = self.link.send::<32>(&KeepAlive.into()).await {
                                break 'main;
                            }
                        }
                        embassy_futures::select::Either::Second(msg) => {
                            let mut payload: Vec<u8, 128> = zenoh_buffers::vec::empty();
                            let mut writer = payload.writer();
                            let codec = Zenoh080::new();
                            codec.write(&mut writer, &msg).unwrap();

                            let Ok(slice) = payload.try_into() else {
                                break 'main;
                            };

                            let frame = Frame {
                                reliability: msg.reliability,
                                sn,
                                ext_qos: zenoh_protocol::transport::frame::ext::QoSType::DEFAULT,
                                payload: slice,
                            };

                            if let Err(_) = self.link.send::<128>(&frame.into()).await {
                                break 'main;
                            }

                            sn += 1;
                        }
                    }
                }
            }
        }
    }
}

pub struct Subscriber {
    receiver: Receiver<'static, NoopRawMutex, ZSlice, 8>,
}

impl Subscriber {
    pub async fn recv(&mut self) -> ZSlice {
        self.receiver.receive().await
    }
}

pub struct SingleLinkClientSession {
    session_to_transport: &'static Channel<NoopRawMutex, NetworkMessage, 8>,
    transport_to_session: &'static Channel<NoopRawMutex, TransportMessage, 8>,

    next_id: u32,

    subscribers: FnvIndexMap<u32, &'static Channel<NoopRawMutex, ZSlice, 8>, 8>,

    mapping: FnvIndexMap<WireExpr<'static, 32>, u32, 16>,
}

impl SingleLinkClientSession {
    pub async fn open<'a, T: Platform>(
        platform: &'a mut T,
        endpoint: EndPoint<32>,
    ) -> ZResult<(Self, SessionRunner<'a, T>)> {
        let mut tm = TransportManager::new(
            platform,
            ZenohIdProto::default(),
            zenoh_protocol::core::WhatAmI::Client,
        );

        zunsafe_arc_pool_init!(ArcBytes256: 1);
        zunsafe_arc_pool_init!(ArcBytes128: 2);
        zunsafe_arc_pool_init!(ArcBytes64: 2);

        static SESSION_TO_TRANSPORT: StaticCell<Channel<NoopRawMutex, NetworkMessage, 8>> =
            StaticCell::new();

        static TRANSPORT_TO_SESSION: StaticCell<Channel<NoopRawMutex, TransportMessage, 8>> =
            StaticCell::new();

        let session_to_transport = SESSION_TO_TRANSPORT.init(Channel::new());
        let transport_to_session = TRANSPORT_TO_SESSION.init(Channel::new());

        let (link, send_open_syn, recv_open_ack) = tm
            .open_transport_link_unicast::<256, _, 32, 32>(&endpoint)
            .await
            .context(zctx!("opening transport link"))?;

        Ok((
            Self {
                session_to_transport,
                transport_to_session,
                next_id: 1,
                subscribers: FnvIndexMap::new(),
                mapping: FnvIndexMap::new(),
            },
            SessionRunner {
                link,
                send_open_syn,
                recv_open_ack,
                tm,
                session_to_transport,
                transport_to_session,
            },
        ))
    }

    async fn internal_update(&mut self, msg: TransportMessage) -> ZResult<()> {
        match msg.body {
            zenoh_protocol::transport::TransportBody::Frame(mut frame) => {
                let codec = (Zenoh080::new(), ZSliceLen::<256>);
                let mut reader = frame.payload.reader();

                let body: NetworkMessage = codec.read(&mut reader)?;

                match body.body {
                    NetworkBody::Push(push) => match push.payload {
                        PushBody::Put(put) => {
                            let id = self.mapping.get(&push.wire_expr).ok_or(zerr!(ZE::Failed))?;
                            let sender = self.subscribers.get_mut(id).ok_or(zerr!(ZE::Failed))?;

                            sender.send(put.payload).await;
                        }
                    },
                    _ => {}
                }
            }
            _ => {}
        }

        Ok(())
    }

    pub async fn read(&mut self) -> ZResult<()> {
        let msg = self.transport_to_session.receive().await;

        self.internal_update(msg).await
    }

    pub async fn try_read(&mut self) -> ZResult<()> {
        let msg = self
            .transport_to_session
            .try_receive()
            .map_err(|_| zerr!(ZE::Failed))?;

        self.internal_update(msg).await
    }

    pub async fn put<const L: usize>(
        &mut self,
        ke: &'static keyexpr,
        bytes: &[u8; L],
    ) -> ZResult<()> {
        if L > 64 {
            bail!(ZE::CapacityExceeded);
        }

        let msg = NetworkMessage {
            reliability: zenoh_protocol::core::Reliability::BestEffort,
            body: NetworkBody::Push(Push {
                wire_expr: WireExpr::from(ke),
                ext_qos: zenoh_protocol::network::ext::QoSType::PUSH,
                ext_tstamp: None,
                ext_nodeid: zenoh_protocol::network::push::ext::NodeIdType::DEFAULT,
                payload: PushBody::Put(Put {
                    timestamp: None,
                    encoding: Encoding::empty(),
                    ext_sinfo: None,
                    ext_attachment: None,
                    ext_unknown: Vec::new(),
                    payload: Vec::<u8, 64>::from_slice(bytes).unwrap().try_into()?,
                }),
            }),
        };

        self.session_to_transport.send(msg).await;

        Ok(())
    }

    pub async fn declare_subscriber<'a>(&'a mut self, ke: &'static keyexpr) -> ZResult<Subscriber> {
        let id = self.next_id;
        self.next_id += 1;

        let ke = WireExpr::from(ke).to_owned()?;
        self.mapping
            .insert(ke.clone(), id)
            .map_err(|_| zerr!(ZE::CapacityExceeded))?;

        static CHANNEL: StaticCell<Channel<NoopRawMutex, ZSlice, 8>> = StaticCell::new();
        let channel = CHANNEL.init(Channel::new());

        self.subscribers
            .insert(id, channel)
            .map_err(|_| zerr!(ZE::CapacityExceeded))?;

        let msg = NetworkMessage {
            reliability: zenoh_protocol::core::Reliability::BestEffort,
            body: NetworkBody::Declare(Declare {
                interest_id: None,
                ext_qos: zenoh_protocol::network::declare::ext::QoSType::DECLARE,
                ext_tstamp: None,
                ext_nodeid: zenoh_protocol::network::declare::ext::NodeIdType::DEFAULT,
                body: DeclareBody::DeclareSubscriber(DeclareSubscriber { id, wire_expr: ke }),
            }),
        };

        self.session_to_transport.send(msg).await;

        Ok(Subscriber {
            receiver: channel.receiver(),
        })
    }
}
