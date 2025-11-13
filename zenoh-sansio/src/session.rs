use core::time::Duration;

use crate::{
    Reliability, ZResult, ZenohIdProto,
    encoding::Encoding,
    ke::keyexpr,
    network::{NetworkBody, NodeId, QoS, push::Push},
    resolution::Resolution,
    transport::{
        TransportBatch, TransportBody,
        batch::Batch,
        init::{BatchSize, InitExt, InitIdentifier, InitResolution, InitSyn},
        open::{OpenExt, OpenSyn},
    },
    whatami::WhatAmI,
    wire_expr::WireExpr,
    zenoh::{PushBody, put::Put},
};

pub(crate) mod establish;

crate::__internal_zerr! {
    /// Errors related to IO operations on byte buffers
    #[err = "session error"]
    enum ZSessionError {
        InvalidArgument,
        PeerKeepAliveTimedOut
    }
}

enum SessionState {
    Disconnected {
        mine: MineConfig,
    },
    Connecting {
        mine: MineConfig,
        negotiated: NegotiatedConfig,

        other_zid: ZenohIdProto,
    },
    Connected {
        mine: MineConfig,
        other: OtherConfig,
        negotiated: NegotiatedConfig,

        next_recv_keepalive: Duration,
        next_send_keepalive: Duration,
    },
}

struct MineConfig {
    mine_zid: ZenohIdProto,
    mine_resolution: Resolution,
    mine_batch_size: u16,

    mine_lease: Duration,
}

struct OtherConfig {
    other_zid: ZenohIdProto,
    other_sn: u32,
    other_lease: Duration,
}

struct NegotiatedConfig {
    negotiated_sn: u32,
    negotiated_resolution: Resolution,
    negotiated_batch_size: u16,
}

pub struct Session {
    state: SessionState,
}

pub fn open() -> (Session, Event<'static>) {
    let mine = MineConfig {
        mine_zid: ZenohIdProto::default(),
        mine_resolution: Resolution::DEFAULT,
        mine_batch_size: u16::MAX,
        mine_lease: Duration::from_secs(10),
    };

    let event = Event::InitSyn(InitSyn {
        version: 9,
        identifier: InitIdentifier {
            whatami: WhatAmI::Client,
            zid: mine.mine_zid.clone(),
        },
        resolution: InitResolution {
            resolution: mine.mine_resolution.clone(),
            batch_size: BatchSize(mine.mine_batch_size),
        },
        ext: InitExt::DEFAULT,
    });

    (
        Session {
            state: SessionState::Disconnected { mine },
        },
        event,
    )
}

impl Session {
    /// Reads bytes from the given buffer, updating the session and returning any resulting event.
    /// (For example, processing an InitAck may result in an OpenSyn event etc...)
    pub fn read<'a>(
        &mut self,
        time: Duration,
        mut v: &'a [u8],
    ) -> ZResult<Event<'a>, ZSessionError> {
        let mut keepalive = false;

        if let Session::Connected {
            next_recv_keepalive,
            next_send_keepalive,
            ..
        } = self
        {
            if v.is_empty() && time >= *next_recv_keepalive {
                *self = Session::Disconnected {
                    zid: ZenohIdProto::default(),
                    resolution: Resolution::DEFAULT,
                    batch_size: u16::MAX,
                    lease: Duration::from_secs(10),
                };

                return Err(ZSessionError::PeerKeepAliveTimedOut);
            }

            if time >= *next_send_keepalive {
                *next_send_keepalive = time + Duration::from_secs(10) / 3;
                keepalive = true;
            }
        }

        let mut batch = TransportBatch::new(&mut v);

        while let Some(msg) = batch.next() {
            match msg {
                TransportBody::Close(_) => {
                    *self = Session::Disconnected {
                        zid: ZenohIdProto::default(),
                        resolution: Resolution::DEFAULT,
                        batch_size: u16::MAX,
                        lease: Duration::from_secs(10),
                    };

                    return Ok(Event::None);
                }
                TransportBody::InitAck(ack) => {
                    if let Session::Disconnected {
                        zid: mine_zid,
                        resolution: mine_resolution,
                        batch_size: mine_batch_size,
                        lease,
                    } = self
                    {
                        let mine_lease = *lease;
                        let other_zid = ack.identifier.zid.clone();

                        let resolution = establish::negotiate_resolution(
                            mine_resolution,
                            &ack.resolution.resolution,
                        )?;

                        let sn = establish::negotiate_sn(mine_zid, &other_zid, &resolution);

                        let batch_size = establish::negotiate_batch_size(
                            *mine_batch_size,
                            ack.resolution.batch_size.0,
                        )?;

                        *self = Session::Connecting {
                            mine_zid: mine_zid.clone(),
                            other_zid: ack.identifier.zid.clone(),
                            lease: mine_lease,

                            sn,
                            resolution,
                            batch_size,
                        };

                        return Ok(Event::OpenSyn(OpenSyn {
                            lease: mine_lease,
                            sn,
                            cookie: ack.cookie,
                            ext: OpenExt::DEFAULT,
                        }));
                    }
                }
                TransportBody::OpenAck(ack) => {
                    if let Session::Connecting {
                        mine_zid,
                        other_zid,
                        lease: mine_lease,
                        sn: mine_sn,
                        resolution,
                        batch_size,
                    } = self
                    {
                        let other_lease = ack.lease;
                        let other_sn = ack.sn;

                        *self = Session::Connected {
                            mine_zid: mine_zid.clone(),
                            other_zid: other_zid.clone(),

                            mine_lease: *mine_lease,
                            other_lease,

                            mine_sn: *mine_sn,
                            other_sn,

                            resolution: *resolution,
                            batch_size: *batch_size,

                            next_recv_keepalive: time + other_lease,
                            next_send_keepalive: time + *mine_lease / 3,
                        };
                    }
                }
                TransportBody::KeepAlive(_) => {
                    if let Session::Connected {
                        next_recv_keepalive,
                        other_lease,
                        ..
                    } = self
                    {
                        *next_recv_keepalive = time + *other_lease;
                    }
                }
                _ => {}
            }
        }

        match keepalive {
            true => Ok(Event::KeepAlive),
            false => Ok(Event::None),
        }
    }

    pub fn dispatch<const N: usize>(
        &mut self,
        tx: &mut [u8],
        events: [Event; N],
        mut dispatch: impl FnMut(&[u8]),
    ) -> ZResult<()> {
        let mut batch = Batch::new(
            &mut tx[..],
            match self {
                Session::Disconnected { .. } => 0,
                Session::Connecting { .. } => 0,
                Session::Connected { mine_sn: sn, .. } => *sn,
            },
        );

        let mut keepalive = false;

        for event in events {
            match event {
                Event::InitSyn(syn) => {
                    batch.write_init_syn(&syn)?;
                }
                Event::OpenSyn(syn) => {
                    batch.write_open_syn(&syn)?;
                }
                Event::KeepAlive => {
                    keepalive = true;
                }
                Event::Push(push) => {
                    if !self.connected() {
                        continue;
                    }

                    batch.write_msg(
                        &NetworkBody::Push(push),
                        Reliability::Reliable,
                        QoS::DEFAULT,
                    )?;
                }
                _ => {}
            }
        }

        if keepalive && !batch.has_written() {
            batch.write_keepalive()?;
        }

        let (sn, len) = batch.finalize();
        if len == 0 {
            return Ok(());
        }

        if let Session::Connected { mine_sn: s, .. } = self {
            *s = sn;
        }

        dispatch(&tx[..len]);

        Ok(())
    }

    pub fn connected(&self) -> bool {
        matches!(self, Session::Connected { .. })
    }

    pub fn disconnected(&self) -> bool {
        matches!(self, Session::Disconnected { .. })
    }

    pub fn lease(&self) -> Duration {
        match self {
            Session::Connected {
                mine_lease: lease, ..
            }
            | Session::Connecting { lease, .. }
            | Session::Disconnected { lease, .. } => *lease / 3,
        }
    }

    pub fn put<'a>(&self, ke: &'a keyexpr, payload: &'a [u8]) -> Event<'a> {
        Event::Push(Push {
            wire_expr: WireExpr::from(ke),
            qos: QoS::DEFAULT,
            timestamp: None,
            nodeid: NodeId::DEFAULT,
            payload: PushBody::Put(Put {
                timestamp: None,
                encoding: Encoding::EMPTY,
                sinfo: None,
                attachment: None,
                payload,
            }),
        })
    }
}

pub enum Event<'a> {
    None,
    InitSyn(InitSyn<'a>),
    OpenSyn(OpenSyn<'a>),
    KeepAlive,
    Push(Push<'a>),
}
