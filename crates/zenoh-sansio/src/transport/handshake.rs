use core::fmt::Display;

use zenoh_proto::TransportError;

use crate::{
    OpenedTransport, TransportRx, TransportTx,
    establishment::{Description, State},
};

pub enum Handshake<Buff, T, Read, Write> {
    Pending {
        #[allow(private_interfaces)]
        state: State,

        streamed: bool,

        tx: TransportTx<Buff>,
        rx: TransportRx<Buff>,

        handle: T,

        read: Read,
        write: Write,
    },
    Ready {
        #[allow(private_interfaces)]
        description: Description,

        streamed: bool,

        tx: TransportTx<Buff>,
        rx: TransportRx<Buff>,
    },
    Opened,
}

pub struct HandshakeReady<'a, Buff, T, Read, Write> {
    handshake: &'a mut Handshake<Buff, T, Read, Write>,
}

impl<'a, Buff, T, Read, Write> HandshakeReady<'a, Buff, T, Read, Write> {
    pub fn open(self) -> OpenedTransport<Buff> {
        if let Handshake::Ready {
            description,
            streamed,
            tx,
            rx,
        } = core::mem::replace(self.handshake, Handshake::Opened)
        {
            OpenedTransport::new(description, streamed, tx.into_inner(), rx.into_inner())
        } else {
            unreachable!()
        }
    }
}

impl<'a, Buff, T, Read, Write> Handshake<Buff, T, Read, Write> {
    pub fn poll<E>(
        &mut self,
    ) -> core::result::Result<Option<HandshakeReady<'_, Buff, T, Read, Write>>, TransportError>
    where
        T: Copy,
        E: Display,
        Buff: Clone + AsMut<[u8]> + AsRef<[u8]>,
        Read: FnMut(&mut T, &mut [u8]) -> core::result::Result<usize, E>,
        Write: FnMut(&mut T, &[u8]) -> core::result::Result<(), E>,
    {
        match self {
            Self::Opened => Ok(None),
            Self::Ready { .. } => Ok(Some(HandshakeReady { handshake: self })),
            Self::Pending {
                state,
                tx,
                rx,
                handle,
                read,
                write,
                ..
            } => {
                if let Some(description) = state.description() {
                    if let Self::Pending {
                        streamed, tx, rx, ..
                    } = core::mem::replace(self, Self::Opened)
                    {
                        *self = Self::Ready {
                            description,
                            streamed,
                            tx,
                            rx,
                        };

                        return Ok(Some(HandshakeReady { handshake: self }));
                    } else {
                        unreachable!()
                    }
                }
                extern crate std;

                rx.decode_with(|bytes| read(handle, bytes))?;
                let resp = rx
                    .flush_t()
                    .map(|msg| {
                        std::println!("Received {:?}", msg.0);
                        state.poll(msg)
                    })
                    .map(|response| {
                        std::println!("Sending {:?}", response.0);
                        response.0
                    })
                    .flatten();
                tx.encode_t(resp);
                if let Some(bytes) = tx.flush() {
                    write(handle, bytes).map_err(|e| {
                        zenoh_proto::error!("{e}");
                        TransportError::CouldNotRead
                    })?;
                }

                Ok(None)
            }
        }
    }
}
