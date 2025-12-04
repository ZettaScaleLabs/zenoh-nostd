use core::{fmt::Debug, time::Duration};

use zenoh_proto::{
    crate::ZCodecError,
    Reliability, Resolution, WhatAmI, ZResult, ZenohIdProto,
    network::{FrameBody, QoS},
    transport::{
        Batch,
        init::{BatchSize, InitExt, InitIdentifier, InitResolution, InitSyn},
    },
};

use crate::event::{Event, EventInner};

pub mod establish;
pub mod event;
pub mod update;

pub mod put;

zenoh_proto::make_zerr! {
    /// Errors related to Zenoh Session
    #[err = "zenoh session error"]
    enum ZSessionError {
        InvalidArgument
    }
}

pub fn open() -> (Session, Event<'static>) {
    let mine = MineConfig {
        mine_zid: ZenohIdProto::default(),
        mine_resolution: Resolution::DEFAULT,
        mine_batch_size: u16::MAX,
        mine_lease: Duration::from_secs(10),
    };

    let event = Event {
        inner: EventInner::InitSyn(InitSyn {
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
        }),
    };

    (
        Session {
            state: SessionState::Disconnected { mine },
        },
        event,
    )
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

#[derive(Clone)]
struct MineConfig {
    mine_zid: ZenohIdProto,
    mine_resolution: Resolution,
    mine_batch_size: u16,

    mine_lease: Duration,
}

#[derive(Clone)]
struct OtherConfig {
    _other_zid: ZenohIdProto,
    _other_sn: u32,
    other_lease: Duration,
}

#[derive(Clone)]
struct NegotiatedConfig {
    negotiated_sn: u32,
    _negotiated_resolution: Resolution,
    _negotiated_batch_size: u16,
}

pub struct Session {
    state: SessionState,
}

impl Session {
    pub fn new() -> Self {
        Self {
            state: SessionState::Disconnected {
                mine: MineConfig {
                    mine_zid: ZenohIdProto::default(),
                    mine_resolution: Resolution::default(),
                    mine_batch_size: 16,
                    mine_lease: Duration::from_secs(60),
                },
            },
        }
    }

    pub fn dispatch<'a, E: Debug>(
        &mut self,
        tx: &mut [u8],
        events: impl Iterator<Item = Event<'a>>,
        mut out: impl FnMut(&[u8]) -> core::result::Result<(), E>,
    ) -> crate::ZResult<()> {
        let mut batch = Batch::new(
            &mut tx[..],
            match &self.state {
                SessionState::Disconnected { .. } => 0,
                SessionState::Connecting { .. } => 0,
                SessionState::Connected { negotiated, .. } => negotiated.negotiated_sn,
            },
        );

        let mut keepalive = false;
        for event in events {
            match event.inner {
                EventInner::InitSyn(syn) => batch.write_init_syn(&syn)?,
                EventInner::OpenSyn(syn) => batch.write_open_syn(&syn)?,
                EventInner::Push(push) => {
                    if !self.connected() {
                        zenoh_proto::warn!("Ignoring Push event while not connected");
                        continue;
                    }

                    batch.write_msg(&FrameBody::Push(push), Reliability::Reliable, QoS::DEFAULT)?
                }
                EventInner::KeepAlive => {
                    keepalive = true;
                }
                _ => {}
            }
        }

        if keepalive && !batch.has_written() {
            zenoh_proto::debug!("Sending keepalive");
            batch.write_keepalive()?;
        }

        let (sn, len) = batch.finalize();
        if len == 0 {
            return Ok(());
        }

        if let SessionState::Connected { negotiated, .. } = &mut self.state {
            negotiated.negotiated_sn = sn;
        }

        match out(&tx[..len]) {
            Ok(_) => Ok(()),
            Err(e) => {
                zenoh_proto::error!("Dispatch error: {:?}", e);
                Ok(())
            }
        }
    }

    pub fn connected(&self) -> bool {
        matches!(self.state, SessionState::Connected { .. })
    }

    pub fn disconnected(&self) -> bool {
        matches!(self.state, SessionState::Disconnected { .. })
    }

    pub fn lease(&self) -> Duration {
        match &self.state {
            SessionState::Disconnected { mine } => mine.mine_lease / 4,
            SessionState::Connecting { mine, .. } => mine.mine_lease / 4,
            SessionState::Connected { mine, .. } => mine.mine_lease / 4,
        }
    }
}

impl From<ZCodecError> for ZSessionError {
    fn from(value: ZCodecError) -> Self {
        match value {
            _ => ZSessionError::InvalidArgument,
        }
    }
}
