use core::fmt::Display;
use core::time::Duration;

use zenoh_proto::{TransportError, ZBodyDecode, ZReadable, fields::Resolution, msgs::*};

use crate::{ZTransportRx, transport::TransportTx};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum State {
    Opened,
    Used,
    Synchronized { last_received: Duration },
    Closed,
}

#[derive(Debug)]
pub struct TransportRx<Buff> {
    buff: Buff,

    cursor: usize,
    batch_size: usize,

    sn: u32,
    resolution: Resolution,
    pub lease: Duration,

    state: State,
}

impl<Buff> TransportRx<Buff> {
    pub(crate) fn new(
        buff: Buff,

        batch_size: usize,
        sn: u32,
        resolution: Resolution,
        lease: Duration,
    ) -> Self {
        Self {
            buff,

            cursor: 0,
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

    pub(crate) fn flush_transport(&mut self) -> impl Iterator<Item = (TransportMessage<'_>, &[u8])>
    where
        Buff: AsMut<[u8]> + AsRef<[u8]>,
    {
        let size = core::cmp::min(
            self.buff.as_ref().len(),
            core::cmp::min(self.batch_size, self.cursor),
        );
        self.cursor = 0;
        let mut reader = &self.buff.as_ref()[..size];
        let mut last_frame = None;
        let sn = &mut self.sn;
        let resolution = self.resolution;

        core::iter::from_fn(move || Self::decode(&mut reader, &mut last_frame, sn, resolution))
            .filter_map(|m| match m.0 {
                Message::Transport(msg) => Some((msg, m.1)),
                _ => None,
            })
    }

    pub fn sync(&mut self, tx: Option<&TransportTx<Buff>>, now: Duration) {
        if let Some(tx) = tx
            && tx.closed()
        {
            self.state = State::Closed;
            return;
        }

        if let State::Synchronized { .. } = self.state
            && now > self.next_timeout()
        {
            self.state = State::Closed;
        }

        if self.state == State::Used {
            self.state = State::Synchronized { last_received: now };
        };
    }

    pub fn next_timeout(&self) -> Duration {
        match self.state {
            State::Opened | State::Closed | State::Used => Duration::from_secs(0).into(),
            State::Synchronized { last_received } => (last_received + self.lease / 4).into(),
        }
    }

    pub fn closed(&self) -> bool {
        matches!(self.state, State::Closed)
    }

    pub(crate) fn decode<'a>(
        reader: &mut &'a [u8],
        last_frame: &mut Option<FrameHeader>,
        sn: &mut u32,
        resolution: Resolution,
    ) -> Option<(Message<'a>, &'a [u8])>
    where
        Buff: AsRef<[u8]>,
    {
        if !reader.can_read() {
            return None;
        }

        let (data, start) = (reader.as_ptr(), reader.len());

        let header = reader
            .read_u8()
            .expect("reader should not be empty at this stage");

        macro_rules! decode {
            ($ty:ty) => {
                match <$ty as ZBodyDecode>::z_body_decode(reader, header) {
                    Ok(msg) => msg,
                    Err(e) => {
                        zenoh_proto::error!(
                            "Failed to decode message of type {}: {}. Skipping the rest of the message - {}",
                            core::any::type_name::<$ty>(),
                            e,
                            zenoh_proto::zctx!()
                        );

                        return None;
                    }
                }
            };

            (@Transport $ty:ident) => {{
                last_frame.take();
                Message::Transport(TransportMessage:: $ty (decode!($ty)))
            }};

            (@Network $ty:ident) => {
                Message::Network(NetworkMessage {
                    reliability: last_frame.as_ref().expect("Should be in frame").reliability,
                    qos: last_frame.as_ref().expect("Should be in frame").qos,
                    body: NetworkBody:: $ty (decode!($ty))
                })
            };
        }

        let ack = header & 0b0010_0000 != 0;
        let net = last_frame.is_some();
        let ifinal = header & 0b0110_0000 == 0;
        let id = header & 0b0001_1111;

        let body = match id {
            FrameHeader::ID => {
                let header = decode!(FrameHeader);

                // Check for missed messages regarding resolution
                let _ = resolution;
                if header.sn <= *sn && *sn != 0 {
                    zenoh_proto::error!(
                        "Inconsistent `SN` value {}, expected higher than {}",
                        header.sn,
                        *sn
                    );
                    return None;
                } else if header.sn != *sn + 1 && *sn != 0 {
                    zenoh_proto::debug!("Transport missed {} messages", header.sn - *sn - 1);
                }

                last_frame.replace(header);
                *sn = header.sn;

                return Self::decode(reader, last_frame, sn, resolution);
            }
            InitAck::ID if ack => decode!(@Transport InitAck),
            InitSyn::ID => decode!(@Transport InitSyn),
            OpenAck::ID if ack => decode!(@Transport OpenAck),
            OpenSyn::ID => decode!(@Transport OpenSyn),
            Close::ID => decode!(@Transport Close),
            KeepAlive::ID => decode!(@Transport KeepAlive),
            Push::ID if net => decode!(@Network Push),
            Request::ID if net => decode!(@Network Request),
            Response::ID if net => decode!(@Network Response),
            ResponseFinal::ID if net => decode!(@Network ResponseFinal),
            InterestFinal::ID if net && ifinal => decode!(@Network InterestFinal),
            Interest::ID if net => decode!(@Network Interest),
            Declare::ID if net => decode!(@Network Declare),
            _ => {
                zenoh_proto::error!(
                    "Unrecognized message header: {:08b}. Skipping the rest of the message - {}",
                    header,
                    zenoh_proto::zctx!()
                );
                return None;
            }
        };

        let len = start - reader.len();
        Some((body, unsafe { core::slice::from_raw_parts(data, len) }))
    }
}

impl<Buff> ZTransportRx for TransportRx<Buff>
where
    Buff: AsMut<[u8]> + AsRef<[u8]>,
{
    fn decode_prefixed(&mut self, mut read: &[u8]) -> core::result::Result<(), TransportError>
    where
        Buff: AsMut<[u8]> + AsRef<[u8]>,
    {
        if read.is_empty() || self.state == State::Closed {
            return Ok(());
        }

        self.decode_prefixed_with(|data| {
            let size = data.len().min(read.len());
            let (ret, remain) = read.split_at(size);
            data[..size].copy_from_slice(ret);
            read = remain;
            Ok::<_, TransportError>(size)
        })
    }

    fn decode_raw(&mut self, read: &[u8]) -> core::result::Result<(), TransportError>
    where
        Buff: AsMut<[u8]> + AsRef<[u8]>,
    {
        if read.is_empty() || self.state == State::Closed {
            return Ok(());
        }

        let max = core::cmp::min(self.buff.as_ref().len(), self.batch_size);
        let buff = &mut self.buff.as_mut()[self.cursor..max];

        let len = read.len();
        if buff.len() < read.len() {
            zenoh_proto::zbail!(@log TransportError::TransportIsFull);
        }

        buff[..len].copy_from_slice(read);

        if len > 0 {
            self.state = State::Used;
        }

        self.cursor += len;

        Ok(())
    }

    fn decode_prefixed_with<E>(
        &mut self,
        mut read: impl FnMut(&mut [u8]) -> core::result::Result<usize, E>,
    ) -> core::result::Result<(), TransportError>
    where
        Buff: AsMut<[u8]> + AsRef<[u8]>,
        E: Display,
    {
        if self.state == State::Closed {
            return Ok(());
        }

        let max = core::cmp::min(self.buff.as_ref().len(), self.batch_size);
        let buff = &mut self.buff.as_mut()[self.cursor..max];

        if 2 > buff.len() {
            zenoh_proto::zbail!(@log TransportError::TransportTooSmall);
        }

        let mut len = [0u8; 2];
        let l = read(&mut len).map_err(|e| {
            zenoh_proto::error!("{e}");
            TransportError::CouldNotRead
        })?;

        if l == 0 {
            return Ok(());
        } else if l != 2 {
            zenoh_proto::zbail!(@log TransportError::InvalidAttribute)
        }

        let len = u16::from_le_bytes(len) as usize;
        if len > u16::MAX as usize || len > buff.len() {
            zenoh_proto::zbail!(@log TransportError::InvalidAttribute);
        }

        if read(&mut buff[..len]).map_err(|e| {
            zenoh_proto::error!("{e}");
            TransportError::CouldNotRead
        })? != len
        {
            zenoh_proto::zbail!(@log TransportError::InvalidAttribute)
        }

        if len > 0 {
            self.state = State::Used;
        }

        self.cursor += len;

        Ok(())
    }

    fn decode_raw_with<E>(
        &mut self,
        mut read: impl FnMut(&mut [u8]) -> core::result::Result<usize, E>,
    ) -> core::result::Result<(), TransportError>
    where
        Buff: AsMut<[u8]> + AsRef<[u8]>,
        E: Display,
    {
        if self.state == State::Closed {
            return Ok(());
        }

        let max = core::cmp::min(self.buff.as_ref().len(), self.batch_size);
        let buff = &mut self.buff.as_mut()[self.cursor..max];

        let len = read(buff).map_err(|e| {
            zenoh_proto::error!("{e}");
            TransportError::CouldNotRead
        })?;

        if len > 0 {
            self.state = State::Used;
        }

        self.cursor += len;

        Ok(())
    }

    async fn decode_prefixed_with_async<E>(
        &mut self,
        mut read: impl AsyncFnMut(&mut [u8]) -> core::result::Result<usize, E>,
    ) -> core::result::Result<(), TransportError>
    where
        Buff: AsMut<[u8]> + AsRef<[u8]>,
        E: Display,
    {
        if self.state == State::Closed {
            return Ok(());
        }

        let max = core::cmp::min(self.buff.as_ref().len(), self.batch_size);
        let buff = &mut self.buff.as_mut()[self.cursor..max];

        if 2 > buff.len() {
            zenoh_proto::zbail!(@log TransportError::TransportTooSmall);
        }

        let mut len = [0u8; 2];
        let l = read(&mut len).await.map_err(|e| {
            zenoh_proto::error!("{e}");
            TransportError::CouldNotRead
        })?;

        if l == 0 {
            return Ok(());
        } else if l != 2 {
            zenoh_proto::zbail!(@log TransportError::InvalidAttribute)
        }

        let len = u16::from_le_bytes(len) as usize;
        if len > u16::MAX as usize || len > buff.len() {
            zenoh_proto::zbail!(@log TransportError::InvalidAttribute);
        }

        if read(&mut buff[..len]).await.map_err(|e| {
            zenoh_proto::error!("{e}");
            TransportError::CouldNotRead
        })? != len
        {
            zenoh_proto::zbail!(@log TransportError::InvalidAttribute)
        }

        if len > 0 {
            self.state = State::Used;
        }

        self.cursor += len;

        Ok(())
    }

    async fn decode_raw_with_async<E>(
        &mut self,
        mut read: impl AsyncFnMut(&mut [u8]) -> core::result::Result<usize, E>,
    ) -> core::result::Result<(), TransportError>
    where
        Buff: AsMut<[u8]> + AsRef<[u8]>,
        E: Display,
    {
        if self.state == State::Closed {
            return Ok(());
        }

        let max = core::cmp::min(self.buff.as_ref().len(), self.batch_size);
        let buff = &mut self.buff.as_mut()[self.cursor..max];

        let len = read(buff).await.map_err(|e| {
            zenoh_proto::error!("{e}");
            TransportError::CouldNotRead
        })?;

        if len > 0 {
            self.state = State::Used;
        }

        self.cursor += len;

        Ok(())
    }

    fn flush(&mut self) -> impl Iterator<Item = (NetworkMessage<'_>, &'_ [u8])>
    where
        Buff: AsMut<[u8]> + AsRef<[u8]>,
    {
        let size = core::cmp::min(
            self.buff.as_ref().len(),
            core::cmp::min(self.batch_size, self.cursor),
        );
        self.cursor = 0;
        let mut reader = &self.buff.as_ref()[..size];
        let mut last_frame = None;
        let sn = &mut self.sn;
        let resolution = self.resolution;

        core::iter::from_fn(move || Self::decode(&mut reader, &mut last_frame, sn, resolution))
            .filter_map(|m| match m.0 {
                Message::Network(msg) => Some((msg, m.1)),
                _ => None,
            })
    }
}
