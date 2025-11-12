use core::{time::Duration, u16};

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

pub enum Session {
    Disconnected {
        zid: ZenohIdProto,
        resolution: Resolution,
        batch_size: u16,

        lease: Duration,
    },
    Connecting {
        mine_zid: ZenohIdProto,
        other_zid: ZenohIdProto,

        lease: Duration,

        sn: u32,
        resolution: Resolution,
        batch_size: u16,
    },
    Connected {
        mine_zid: ZenohIdProto,
        other_zid: ZenohIdProto,

        mine_lease: Duration,
        other_lease: Duration,

        mine_sn: u32,
        other_sn: u32,

        resolution: Resolution,
        batch_size: u16,

        next_recv_keepalive: Duration,
        next_send_keepalive: Duration,
    },
}

pub fn open() -> (Session, Event<'static>) {
    let zid = ZenohIdProto::default();
    let resolution = Resolution::DEFAULT;
    let batch_size = u16::MAX;
    let lease = Duration::from_secs(10);

    (
        Session::Disconnected {
            zid: zid.clone(),
            resolution: resolution.clone(),
            batch_size: batch_size.clone(),
            lease: lease.clone(),
        },
        Event::InitSyn(InitSyn {
            version: 9,
            identifier: InitIdentifier {
                whatami: WhatAmI::Client,
                zid: zid,
            },
            resolution: InitResolution {
                resolution,
                batch_size: BatchSize(batch_size),
            },
            ext: InitExt::DEFAULT,
        }),
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
                        let mine_lease = lease.clone();
                        let other_zid = ack.identifier.zid.clone();

                        let resolution = establish::negotiate_resolution(
                            mine_resolution,
                            &ack.resolution.resolution,
                        )?;

                        let sn = establish::negotiate_sn(&mine_zid, &other_zid, &resolution);

                        let batch_size = establish::negotiate_batch_size(
                            *mine_batch_size,
                            ack.resolution.batch_size.0,
                        )?;

                        *self = Session::Connecting {
                            mine_zid: mine_zid.clone(),
                            other_zid: ack.identifier.zid.clone(),
                            lease: mine_lease.clone(),

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

                            resolution: resolution.clone(),
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
                    keepalive = false;
                }
                Event::OpenSyn(syn) => {
                    batch.write_open_syn(&syn)?;
                    keepalive = false;
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
                    keepalive = false;
                }
                _ => {}
            }
        }

        if keepalive {
            batch.write_keepalive()?;
        }

        // if empty {
        //     if let Session::Connected {
        //         next_send_keepalive,
        //         mine_lease,
        //         last_time,
        //         ..
        //     } = self
        //     {
        //         if *last_time >= *next_send_keepalive {
        //             *next_send_keepalive = *last_time + *mine_lease / 3;
        //             extern crate std;
        //             std::println!("Sending KeepAlive at {:?}", *last_time);
        //             batch.write_keepalive()?;
        //         }
        //     }
        // }

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
