use embassy_executor::Spawner;
use embassy_futures::select::select;
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Channel};
use embassy_time::Timer;
use zenoh_buffers::writer::HasWriter;
use zenoh_codec::{WCodec, Zenoh080};
use zenoh_protocol::{
    core::{key_expr::keyexpr, Encoding, EndPoint, WireExpr, ZenohIdProto},
    network::{ext::NodeIdType, NetworkBody, NetworkMessage, Push},
    transport::{ext::QoSType, Frame, KeepAlive, TransportMessage},
    zenoh::{PushBody, Put},
};
use zenoh_result::{zerror, ZResult};
use zenoh_transport::{
    unicast::{
        link::TransportLinkUnicast,
        open::{RecvOpenAckOut, SendOpenSynOut},
    },
    TransportManager,
};

static SESSION_TO_TRANSPORT: Channel<ThreadModeRawMutex, NetworkMessage, 8> = Channel::new();
static TRANSPORT_TO_SESSION: Channel<ThreadModeRawMutex, TransportMessage, 8> = Channel::new();

pub struct SingleLinkClientSession {}

#[embassy_executor::task]
async fn link_task(
    link: TransportLinkUnicast,
    send_open_syn: SendOpenSynOut,
    recv_open_ack: RecvOpenAckOut,
    tm: TransportManager,
) {
    let mut sn = send_open_syn.mine_initial_sn + 1;

    let mut link = link;
    let keep_alive_timeout = tm.unicast.lease / (tm.unicast.keep_alive as u32);
    let other_lease = recv_open_ack.other_lease;

    let mut last_read_time = embassy_time::Instant::now();
    let mut last_write_time = embassy_time::Instant::now();

    'main: loop {
        let read_lease = Timer::at(last_read_time + other_lease.try_into().unwrap());
        let write_lease = Timer::at(last_write_time + keep_alive_timeout.try_into().unwrap());

        match select(
            select(read_lease, link.recv()),
            select(write_lease, SESSION_TO_TRANSPORT.receive()),
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
                            TRANSPORT_TO_SESSION.send(msg).await;
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
                        if let Err(_) = link.send(&KeepAlive.into()).await {
                            break 'main;
                        }
                    }
                    embassy_futures::select::Either::Second(msg) => {
                        let mut payload: Vec<u8> = Vec::with_capacity(64);
                        let mut writer = payload.writer();
                        let codec = Zenoh080::new();
                        codec.write(&mut writer, &msg).unwrap();

                        let frame = Frame {
                            reliability: msg.reliability,
                            sn,
                            ext_qos: QoSType::DEFAULT,
                            payload: payload.into(),
                        };

                        if let Err(_) = link.send(&frame.into()).await {
                            break 'main;
                        }

                        sn += 1;
                    }
                }
            }
        }
    }
}

impl SingleLinkClientSession {
    pub async fn open(endpoint: EndPoint, spawner: Spawner) -> ZResult<Self> {
        let tm = TransportManager::new(
            ZenohIdProto::default(),
            zenoh_protocol::core::WhatAmI::Client,
        );

        let (link, send_open_syn, recv_open_ack) =
            tm.open_transport_link_unicast(&endpoint).await?;

        spawner
            .spawn(link_task(link, send_open_syn, recv_open_ack, tm))
            .map_err(|_| zerror!("Failed to spawn link task"))?;

        Ok(SingleLinkClientSession {})
    }

    pub async fn read(&mut self) -> ZResult<()> {
        let _ = TRANSPORT_TO_SESSION.receive().await;

        Ok(())
    }

    pub fn try_read(&mut self) -> ZResult<()> {
        let _ = TRANSPORT_TO_SESSION.try_receive();

        Ok(())
    }

    pub async fn put(&mut self, keyexpr: &'static keyexpr, bytes: &[u8]) -> ZResult<()> {
        let msg = NetworkMessage {
            reliability: zenoh_protocol::core::Reliability::BestEffort,
            body: NetworkBody::Push(Push {
                wire_expr: WireExpr::from(keyexpr),
                ext_qos: zenoh_protocol::network::ext::QoSType::DEFAULT,
                ext_tstamp: None,
                ext_nodeid: NodeIdType::DEFAULT,
                payload: PushBody::Put(Put {
                    timestamp: None,
                    encoding: Encoding::empty(),
                    ext_sinfo: None,
                    ext_attachment: None,
                    ext_unknown: vec![],
                    payload: bytes.to_vec().into(),
                }),
            }),
        };

        SESSION_TO_TRANSPORT.send(msg).await;

        Ok(())
    }
}
