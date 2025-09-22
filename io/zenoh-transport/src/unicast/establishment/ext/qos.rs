use async_trait::async_trait;
use core::marker::PhantomData;
use core::str::FromStr;
use std::boxed::Box;
use zenoh_buffers::{
    reader::{DidntRead, Reader},
    writer::{DidntWrite, Writer},
};
use zenoh_codec::{RCodec, WCodec, Zenoh080};
use zenoh_protocol::{
    core::{EndPoint, Metadata, Priority, PriorityRange, Reliability},
    transport::{init, open},
};
use zenoh_result::{zerror, Error as ZError, ZResult};

use crate::unicast::establishment::{AcceptFsm, OpenFsm};

// Extension Fsm
pub(crate) struct QoSFsm<'a> {
    _a: PhantomData<&'a ()>,
}

impl QoSFsm<'_> {
    pub(crate) const fn new() -> Self {
        Self { _a: PhantomData }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum State {
    NoQoS,
    QoS {
        reliability: Option<Reliability>,
        priorities: Option<PriorityRange>,
    },
}

impl State {
    fn new(is_qos: bool, endpoint: &EndPoint) -> ZResult<Self> {
        if !is_qos {
            Ok(State::NoQoS)
        } else {
            let metadata = endpoint.metadata();

            let reliability = metadata
                .get(Metadata::RELIABILITY)
                .map(Reliability::from_str)
                .transpose()
                .map_err(|e| {
                    zerror!("Failed to parse Reliability from EndPoint metadata: {}", e)
                })?;

            let priorities = metadata
                .get(Metadata::PRIORITIES)
                .map(PriorityRange::from_str)
                .transpose()
                .map_err(|e| {
                    zerror!(
                        "Failed to parse PriorityRange from EndPoint metadata: {}",
                        e
                    )
                })?;

            Ok(State::QoS {
                priorities,
                reliability,
            })
        }
    }

    fn try_from_u64(value: u64) -> ZResult<Self> {
        match value {
            0b000_u64 => Ok(State::NoQoS),
            0b001_u64 => Ok(State::QoS {
                priorities: None,
                reliability: None,
            }),
            value if value & 0b110_u64 != 0 => {
                let tag = value & 0b111_u64;

                let priorities = if tag & 0b010_u64 != 0 {
                    let start = Priority::try_from(((value >> 3) & 0xff) as u8)?;
                    let end = Priority::try_from(((value >> (3 + 8)) & 0xff) as u8)?;

                    Some(PriorityRange::new(start..=end))
                } else {
                    None
                };

                let reliability = if tag & 0b100_u64 != 0 {
                    let bit = ((value >> (3 + 8 + 8)) & 0x1) as u8 == 1;

                    Some(Reliability::from(bit))
                } else {
                    None
                };

                Ok(State::QoS {
                    priorities,
                    reliability,
                })
            }
            _ => Err(zerror!("invalid QoS").into()),
        }
    }

    /// Encodes [`QoS`] as a [`u64`].
    ///
    /// This function is used for encoding both of [`StateAccept`] in
    /// [`crate::unicast::establishment::cookie::Cookie::ext_qos`] and
    /// [`zenoh_protocol::transport::init::ext::QoS`].
    ///
    /// The three least significant bits are used to discrimnate five states:
    ///
    /// 1. QoS is disabled
    /// 2. QoS is enabled but no priority range and no reliability setting are available
    /// 3. QoS is enabled and priority range is available but reliability is unavailable
    /// 4. QoS is enabled and reliability is available but priority range is unavailable
    /// 5. QoS is enabled and both priority range and reliability are available
    fn to_u64(&self) -> u64 {
        match self {
            State::NoQoS => 0b000_u64,
            State::QoS {
                priorities,
                reliability,
            } => {
                if reliability.is_none() && priorities.is_none() {
                    return 0b001_u64;
                }

                let mut value = 0b000_u64;

                if let Some(priorities) = priorities {
                    value |= 0b010_u64;
                    value |= (*priorities.start() as u64) << 3;
                    value |= (*priorities.end() as u64) << (3 + 8);
                }

                if let Some(reliability) = reliability {
                    value |= 0b100_u64;
                    value |= (bool::from(*reliability) as u64) << (3 + 8 + 8);
                }

                value
            }
        }
    }

    fn to_exts(&self) -> (Option<init::ext::QoS>, Option<init::ext::QoSLink>) {
        match self {
            State::NoQoS => (None, None),
            State::QoS {
                reliability: None,
                priorities: None,
            } => (Some(init::ext::QoS::new()), None),
            State::QoS {
                reliability: Some(_),
                ..
            }
            | State::QoS {
                priorities: Some(_),
                ..
            } => (None, Some(init::ext::QoSLink::new(self.to_u64()))),
        }
    }

    fn try_from_exts(
        (qos, qos_link): (Option<init::ext::QoS>, Option<init::ext::QoSLink>),
    ) -> ZResult<Self> {
        match (qos, qos_link) {
            (Some(_), Some(_)) => Err(zerror!(
                "Extensions QoS and QoSOptimized cannot both be enabled at once"
            )
            .into()),
            (Some(_), None) => Ok(State::QoS {
                reliability: None,
                priorities: None,
            }),
            (None, Some(qos)) => State::try_from_u64(qos.value),
            (None, None) => Ok(State::NoQoS),
        }
    }

    fn is_qos(&self) -> bool {
        matches!(self, State::QoS { .. })
    }

    fn priorities(&self) -> Option<PriorityRange> {
        match self {
            State::NoQoS
            | State::QoS {
                priorities: None, ..
            } => None,
            State::QoS {
                priorities: Some(priorities),
                ..
            } => Some(priorities.clone()),
        }
    }

    fn reliability(&self) -> Option<Reliability> {
        match self {
            State::NoQoS
            | State::QoS {
                reliability: None, ..
            } => None,
            State::QoS {
                reliability: Some(reliability),
                ..
            } => Some(*reliability),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct StateOpen(State);

impl From<State> for StateOpen {
    fn from(value: State) -> Self {
        StateOpen(value)
    }
}

impl StateOpen {
    pub(crate) fn new(is_qos: bool, endpoint: &EndPoint) -> ZResult<Self> {
        State::new(is_qos, endpoint).map(StateOpen)
    }

    pub(crate) fn is_qos(&self) -> bool {
        self.0.is_qos()
    }

    pub(crate) fn priorities(&self) -> Option<PriorityRange> {
        self.0.priorities()
    }

    pub(crate) fn reliability(&self) -> Option<Reliability> {
        self.0.reliability()
    }
}

#[async_trait]
impl<'a> OpenFsm for &'a QoSFsm<'a> {
    type Error = ZError;

    type SendInitSynIn = &'a StateOpen;
    type SendInitSynOut = (Option<init::ext::QoS>, Option<init::ext::QoSLink>);
    async fn send_init_syn(
        self,
        state: Self::SendInitSynIn,
    ) -> Result<Self::SendInitSynOut, Self::Error> {
        Ok(state.0.to_exts())
    }

    type RecvInitAckIn = (
        &'a mut StateOpen,
        (Option<init::ext::QoS>, Option<init::ext::QoSLink>),
    );
    type RecvInitAckOut = ();
    async fn recv_init_ack(
        self,
        input: Self::RecvInitAckIn,
    ) -> Result<Self::RecvInitAckOut, Self::Error> {
        let (state_self, other_ext) = input;

        let state_other = State::try_from_exts(other_ext)?;

        let (
            State::QoS {
                reliability: self_reliability,
                priorities: self_priorities,
            },
            State::QoS {
                reliability: other_reliability,
                priorities: other_priorities,
            },
        ) = (state_self.0.clone(), state_other)
        else {
            *state_self = State::NoQoS.into();
            return Ok(());
        };

        let priorities = match (self_priorities, other_priorities) {
            (None, priorities) | (priorities, None) => priorities,
            (Some(self_priorities), Some(other_priorities)) => {
                if other_priorities.includes(&self_priorities) {
                    Some(self_priorities)
                } else {
                    return Err(zerror!(
                        "The PriorityRange received in InitAck is not a superset of my PriorityRange"
                    )
                    .into());
                }
            }
        };

        let reliability = match (self_reliability, other_reliability) {
            (None, reliability) | (reliability, None) => reliability,
            (Some(self_reliability), Some(other_reliability)) => {
                if self_reliability == other_reliability {
                    Some(self_reliability)
                } else {
                    return Err(zerror!(
                        "The Reliability received in InitAck doesn't match my Reliability"
                    )
                    .into());
                }
            }
        };

        *state_self = State::QoS {
            reliability,
            priorities,
        }
        .into();

        Ok(())
    }

    type SendOpenSynIn = &'a StateOpen;
    type SendOpenSynOut = Option<open::ext::QoS>;
    async fn send_open_syn(
        self,
        _state: Self::SendOpenSynIn,
    ) -> Result<Self::SendOpenSynOut, Self::Error> {
        Ok(None)
    }

    type RecvOpenAckIn = (&'a mut StateOpen, Option<open::ext::QoS>);
    type RecvOpenAckOut = ();
    async fn recv_open_ack(
        self,
        _state: Self::RecvOpenAckIn,
    ) -> Result<Self::RecvOpenAckOut, Self::Error> {
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct StateAccept(State);

impl From<State> for StateAccept {
    fn from(value: State) -> Self {
        StateAccept(value)
    }
}

impl StateAccept {
    pub(crate) fn new(is_qos: bool, endpoint: &EndPoint) -> ZResult<Self> {
        State::new(is_qos, endpoint).map(StateAccept::from)
    }

    pub(crate) fn is_qos(&self) -> bool {
        self.0.is_qos()
    }

    pub(crate) fn priorities(&self) -> Option<PriorityRange> {
        self.0.priorities()
    }

    pub(crate) fn reliability(&self) -> Option<Reliability> {
        self.0.reliability()
    }
}

// Codec
impl<W> WCodec<&StateAccept, &mut W> for Zenoh080
where
    W: Writer,
{
    type Output = Result<(), DidntWrite>;

    fn write(self, writer: &mut W, x: &StateAccept) -> Self::Output {
        self.write(writer, &x.0.to_u64())
    }
}

impl<R> RCodec<StateAccept, &mut R> for Zenoh080
where
    R: Reader,
{
    type Error = DidntRead;

    fn read(self, reader: &mut R) -> Result<StateAccept, Self::Error> {
        Ok(State::try_from_u64(self.read(reader)?)
            .map_err(|_| DidntRead)?
            .into())
    }
}

#[async_trait]
impl<'a> AcceptFsm for &'a QoSFsm<'a> {
    type Error = ZError;

    type RecvInitSynIn = (
        &'a mut StateAccept,
        (Option<init::ext::QoS>, Option<init::ext::QoSLink>),
    );
    type RecvInitSynOut = ();
    async fn recv_init_syn(
        self,
        input: Self::RecvInitSynIn,
    ) -> Result<Self::RecvInitSynOut, Self::Error> {
        let (state_self, other_ext) = input;

        let state_other = State::try_from_exts(other_ext)?;

        let (
            State::QoS {
                reliability: self_reliability,
                priorities: self_priorities,
            },
            State::QoS {
                reliability: other_reliability,
                priorities: other_priorities,
            },
        ) = (state_self.0.clone(), state_other)
        else {
            *state_self = State::NoQoS.into();
            return Ok(());
        };

        let priorities = match (self_priorities, other_priorities) {
            (None, priorities) | (priorities, None) => priorities,
            (Some(self_priorities), Some(other_priorities)) => {
                if self_priorities.includes(&other_priorities) {
                    Some(other_priorities)
                } else {
                    return Err(zerror!(
                        "The PriorityRange received in InitSyn is not a subset of my PriorityRange"
                    )
                    .into());
                }
            }
        };

        let reliability = match (self_reliability, other_reliability) {
            (None, reliability) | (reliability, None) => reliability,
            (Some(self_reliability), Some(other_reliability)) => {
                if self_reliability == other_reliability {
                    Some(other_reliability)
                } else {
                    return Err(zerror!(
                        "The Reliability received in InitSyn doesn't match my Reliability"
                    )
                    .into());
                }
            }
        };

        *state_self = State::QoS {
            reliability,
            priorities,
        }
        .into();

        Ok(())
    }

    type SendInitAckIn = &'a StateAccept;
    type SendInitAckOut = (Option<init::ext::QoS>, Option<init::ext::QoSLink>);
    async fn send_init_ack(
        self,
        state: Self::SendInitAckIn,
    ) -> Result<Self::SendInitAckOut, Self::Error> {
        Ok(state.0.to_exts())
    }

    type RecvOpenSynIn = (&'a mut StateAccept, Option<open::ext::QoS>);
    type RecvOpenSynOut = ();
    async fn recv_open_syn(
        self,
        _state: Self::RecvOpenSynIn,
    ) -> Result<Self::RecvOpenSynOut, Self::Error> {
        Ok(())
    }

    type SendOpenAckIn = &'a StateAccept;
    type SendOpenAckOut = Option<open::ext::QoS>;
    async fn send_open_ack(
        self,
        _state: Self::SendOpenAckIn,
    ) -> Result<Self::SendOpenAckOut, Self::Error> {
        Ok(None)
    }
}
