use core::fmt::Display;
use core::time::Duration;

use establishment::Description;
use zenoh_proto::{TransportError, ZInstant, fields::*, msgs::*};

pub(crate) mod establishment;

mod handshake;
mod rx;
mod tx;

pub use handshake::*;
pub use rx::*;
pub use tx::*;

use crate::transport::establishment::State;

pub struct Transport<Buff> {
    zid: ZenohIdProto,
    streamed: bool,
    batch_size: u16,
    lease: Duration,
    resolution: Resolution,

    buff: Buff,
}

impl<Buff> Transport<Buff> {
    pub fn new(buff: Buff) -> Self
    where
        Buff: AsRef<[u8]>,
    {
        Transport {
            zid: ZenohIdProto::default(),
            streamed: false,
            batch_size: buff.as_ref().len() as u16,
            lease: Duration::from_secs(10),
            resolution: Resolution::default(),
            buff: buff,
        }
    }
    pub fn with_zid(mut self, zid: ZenohIdProto) -> Self {
        self.zid = zid;
        self
    }

    pub fn streamed(mut self) -> Self {
        self.streamed = true;
        self
    }

    pub fn with_batch_size(mut self, batch_size: u16) -> Self {
        self.batch_size = batch_size;
        self
    }

    pub fn with_lease(mut self, lease: Duration) -> Self {
        self.lease = lease;
        self
    }

    pub fn with_resolution(mut self, resolution: Resolution) -> Self {
        self.resolution = resolution;
        self
    }

    pub fn with_buff<NewBuff>(self, buff: NewBuff) -> Transport<NewBuff> {
        Transport {
            zid: self.zid,
            streamed: self.streamed,
            batch_size: self.batch_size,
            lease: self.lease,
            resolution: self.resolution,
            buff,
        }
    }

    pub fn codec(self) -> OpenedTransport<Buff>
    where
        Buff: Clone,
    {
        OpenedTransport {
            tx: TransportTx::new(
                self.buff.clone(),
                self.streamed,
                self.batch_size as usize,
                0,
                self.resolution,
                self.lease,
            ),
            rx: TransportRx::new(
                self.buff,
                self.streamed,
                self.batch_size as usize,
                0,
                self.resolution,
                self.lease,
            ),
            mine_zid: self.zid,
            other_zid: self.zid,
        }
    }

    pub fn listen<'a, T, E, Read, Write>(
        self,
        handle: T,
        read: Read,
        write: Write,
    ) -> Handshake<Buff, T, Read, Write>
    where
        E: Display,
        Buff: Clone + AsMut<[u8]> + AsRef<[u8]>,
        Read: FnMut(&mut T, &mut [u8]) -> core::result::Result<usize, E>,
        Write: FnMut(&mut T, &[u8]) -> core::result::Result<(), E>,
    {
        let state = State::WaitingInitSyn {
            mine_zid: self.zid,
            mine_batch_size: self.batch_size,
            mine_resolution: self.resolution,
            mine_lease: self.lease,
        };

        let tx = TransportTx::new(
            self.buff.clone(),
            self.streamed,
            self.batch_size as usize,
            0,
            self.resolution,
            self.lease,
        );

        let rx = TransportRx::new(
            self.buff,
            self.streamed,
            self.batch_size as usize,
            0,
            self.resolution,
            self.lease,
        );

        Handshake::Pending {
            state,
            streamed: self.streamed,
            tx,
            rx,
            handle,
            read,
            write,
        }

        // let description = loop {
        //     if let Some(description) = state.description() {
        //         break description;
        //     }

        //     rx.decode_with(|bytes| read(handle, bytes))?;
        //     let resp = rx
        //         .flush_t()
        //         .map(|msg| state.poll(msg))
        //         .map(|response| response.0)
        //         .flatten();
        //     tx.encode_t(resp);
        //     if let Some(bytes) = tx.flush() {
        //         write(handle, bytes).map_err(|e| {
        //             zenoh_proto::error!("{e}");
        //             TransportError::CouldNotRead
        //         })?;
        //     }
        // };

        // let (tx, rx) = (tx.into_inner(), rx.into_inner());

        // Ok(OpenedTransport::new(description, self.streamed, tx, rx))
    }

    pub fn connect<T, E, Read, Write>(
        self,
        mut handle: T,
        read: Read,
        mut write: Write,
    ) -> core::result::Result<Handshake<Buff, T, Read, Write>, TransportError>
    where
        E: Display,
        Buff: Clone + AsMut<[u8]> + AsRef<[u8]>,
        Read: FnMut(&mut T, &mut [u8]) -> core::result::Result<usize, E>,
        Write: FnMut(&mut T, &[u8]) -> core::result::Result<(), E>,
    {
        let state = State::WaitingInitAck {
            mine_zid: self.zid,
            mine_batch_size: self.batch_size,
            mine_resolution: self.resolution,
            mine_lease: self.lease,
        };

        let mut tx = TransportTx::new(
            self.buff.clone(),
            self.streamed,
            self.batch_size as usize,
            0,
            self.resolution,
            self.lease,
        );

        let rx = TransportRx::new(
            self.buff,
            self.streamed,
            self.batch_size as usize,
            0,
            self.resolution,
            self.lease,
        );

        tx.encode_t(core::iter::once(TransportMessage::InitSyn(InitSyn {
            identifier: InitIdentifier {
                zid: self.zid,
                ..Default::default()
            },
            resolution: InitResolution {
                resolution: self.resolution,
                batch_size: BatchSize(self.batch_size),
            },
            ..Default::default()
        })));

        if let Some(bytes) = tx.flush() {
            write(&mut handle, bytes).map_err(|e| {
                zenoh_proto::error!("{e}");
                TransportError::CouldNotRead
            })?;
        }

        Ok(Handshake::Pending {
            state,
            streamed: self.streamed,
            tx,
            rx,
            handle,
            read,
            write,
        })

        // let description = loop {
        //     if let Some(description) = state.description() {
        //         break description;
        //     }

        //     rx.decode_with(|bytes| read(handle, bytes))?;
        //     let resp = rx
        //         .flush_t()
        //         .map(|msg| state.poll(msg))
        //         .map(|response| response.0)
        //         .flatten();

        //     tx.encode_t(resp);
        //     if let Some(bytes) = tx.flush() {
        //         write(handle, bytes).map_err(|e| {
        //             zenoh_proto::error!("{e}");
        //             TransportError::CouldNotRead
        //         })?;
        //     }
        // };

        // let (tx, rx) = (tx.into_inner(), rx.into_inner());

        // Ok(OpenedTransport::new(description, self.streamed, tx, rx))
    }
}

pub struct OpenedTransport<Buff> {
    pub tx: TransportTx<Buff>,
    pub rx: TransportRx<Buff>,

    pub mine_zid: ZenohIdProto,
    pub other_zid: ZenohIdProto,
}

impl<Buff> OpenedTransport<Buff> {
    pub(crate) fn new(description: Description, streamed: bool, tx: Buff, rx: Buff) -> Self {
        Self {
            tx: TransportTx::new(
                tx,
                streamed,
                description.batch_size as usize,
                description.mine_sn,
                description.resolution,
                description.mine_lease,
            ),
            rx: TransportRx::new(
                rx,
                streamed,
                description.batch_size as usize,
                description.other_sn,
                description.resolution,
                description.other_lease,
            ),
            mine_zid: description.mine_zid,
            other_zid: description.other_zid,
        }
    }

    pub fn sync(&mut self, now: ZInstant) {
        let Self { tx, rx, .. } = self;
        rx.sync(&tx, now);
        tx.sync(&rx, now);
    }
}
