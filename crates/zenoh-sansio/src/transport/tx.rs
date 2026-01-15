use core::time::Duration;

use zenoh_proto::{
    TransportError, ZInstant,
    fields::Resolution,
    msgs::{NetworkMessage, TransportMessage},
};

use crate::transport::TransportRx;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum State {
    Opened,
    Used,
    Synchronized { last_received: ZInstant },
    Closed,
}

#[derive(Debug)]
pub struct TransportTx<Buff> {
    buff: Buff,
    streamed: bool,
    cursor: usize,
    batch_size: usize,

    sn: u32,
    resolution: Resolution,
    lease: Duration,

    state: State,
}

impl<Buff> TransportTx<Buff> {
    pub(crate) fn new(
        buff: Buff,

        streamed: bool,
        batch_size: usize,
        sn: u32,
        resolution: Resolution,
        lease: Duration,
    ) -> Self {
        Self {
            buff,
            streamed,
            cursor: if streamed { 2 } else { 0 },
            batch_size,
            sn,
            resolution,
            lease,
            state: State::Opened,
        }
    }

    pub(crate) fn into_inner(self) -> Buff {
        self.buff
    }

    pub(crate) fn encode_t<'a>(&mut self, msg: impl Iterator<Item = TransportMessage<'a>>)
    where
        Buff: AsMut<[u8]> + AsRef<[u8]>,
    {
        let max = core::cmp::min(self.buff.as_ref().len(), self.batch_size);
        let buff = &mut self.buff.as_mut()[self.cursor..max];

        let len = zenoh_proto::transport_encoder(buff, msg).sum::<usize>();

        if len != 0 {
            self.state = State::Used;
        }

        self.cursor += len;
    }

    pub fn encode<'a>(&mut self, msgs: impl Iterator<Item = NetworkMessage<'a>>)
    where
        Buff: AsMut<[u8]> + AsRef<[u8]>,
    {
        let max = core::cmp::min(self.buff.as_ref().len(), self.batch_size);
        let buff = &mut self.buff.as_mut()[self.cursor..max];

        let len =
            zenoh_proto::network_encoder(buff, msgs, &mut self.sn, self.resolution).sum::<usize>();

        if len != 0 {
            self.state = State::Used;
        }

        self.cursor += len;
    }

    pub fn encode_ref<'a>(&mut self, msgs: impl Iterator<Item = &'a NetworkMessage<'a>>)
    where
        Buff: AsMut<[u8]> + AsRef<[u8]>,
    {
        let max = core::cmp::min(self.buff.as_ref().len(), self.batch_size);
        let buff = &mut self.buff.as_mut()[self.cursor..max];

        let len = zenoh_proto::network_encoder_ref(buff, msgs, &mut self.sn, self.resolution)
            .sum::<usize>();

        if len != 0 {
            self.state = State::Used;
        }

        self.cursor += len;
    }

    pub fn flush(&mut self) -> Option<&'_ [u8]>
    where
        Buff: AsMut<[u8]> + AsRef<[u8]>,
    {
        let size = core::cmp::min(
            self.buff.as_ref().len(),
            core::cmp::min(self.batch_size, self.cursor),
        );

        if self.streamed {
            if size < 2 {
                zenoh_proto::zbail!(@None TransportError::TransportTooSmall);
            }

            let len = ((size - 2) as u16).to_le_bytes();
            self.buff.as_mut()[..2].copy_from_slice(&len);
            self.cursor = 2;
        } else {
            self.cursor = 0;
        }

        let buff_ref = &self.buff.as_ref()[..size];
        if size > 0 { Some(buff_ref) } else { None }
    }

    pub fn sync(&mut self, rx: &TransportRx<Buff>, now: ZInstant) {
        if rx.closed() {
            self.state = State::Closed;
            return;
        }

        if let State::Synchronized { .. } = self.state {
            if now.0 > self.next_timeout().0 {
                self.state = State::Closed;
            }
        }

        if self.state == State::Used {
            self.state = State::Synchronized { last_received: now };
        };
    }

    pub fn next_timeout(&self) -> ZInstant {
        match self.state {
            State::Opened | State::Closed | State::Used => Duration::from_secs(0).into(),
            State::Synchronized { last_received } => (last_received.0 + self.lease / 4).into(),
        }
    }

    pub fn closed(&self) -> bool {
        matches!(self.state, State::Closed)
    }
}
