use {
    core::net::SocketAddr,
    futures_util::{
        SinkExt as _, StreamExt as _,
        stream::{SplitSink, SplitStream},
    },
    std::marker::PhantomData,
    zenoh_nostd::{
        ZResult,
        platform::{
            ZConnectionError,
            ws::{AbstractedWsRx, AbstractedWsStream, AbstractedWsTx},
        },
        zbail,
    },
};

#[cfg(all(feature = "async_wsocket", not(feature = "yawc")))]
use async_wsocket::{Message, WebSocket};

#[cfg(feature = "yawc")]
use yawc::{
    WebSocket,
    frame::{FrameView, OpCode},
};

pub struct WasmWsStream {
    pub peer_addr: SocketAddr,
    pub socket: WebSocket,
    mtu: u16,
}

impl WasmWsStream {
    pub fn new(peer_addr: SocketAddr, socket: WebSocket) -> Self {
        Self {
            peer_addr,
            socket,
            mtu: u16::MAX,
        }
    }
}

#[cfg(all(feature = "async_wsocket", not(feature = "yawc")))]
pub struct WasmWsTx<'a> {
    pub socket: SplitSink<WebSocket, Message>,
    _phantom: PhantomData<&'a ()>,
}

#[cfg(feature = "yawc")]
pub struct WasmWsTx<'a> {
    pub socket: SplitSink<WebSocket, FrameView>,
    _phantom: PhantomData<&'a ()>,
}

pub struct WasmWsRx<'a> {
    pub socket: SplitStream<WebSocket>,
    _phantom: PhantomData<&'a ()>,
}

impl AbstractedWsStream for WasmWsStream {
    type Tx<'a> = WasmWsTx<'a>;
    type Rx<'a> = WasmWsRx<'a>;

    fn mtu(&self) -> u16 {
        self.mtu
    }

    fn split(&mut self) -> (Self::Tx<'_>, Self::Rx<'_>) {
        let (tx, rx) = self.socket.clone().split();
        let tx = WasmWsTx {
            socket: tx,
            _phantom: PhantomData,
        };
        let rx = WasmWsRx {
            socket: rx,
            _phantom: PhantomData,
        };
        (tx, rx)
    }

    async fn write(&mut self, buffer: &[u8]) -> ZResult<usize, ZConnectionError> {
        #[cfg(all(feature = "async_wsocket", not(feature = "yawc")))]
        let item = Message::Binary(buffer.to_vec());
        #[cfg(feature = "yawc")]
        let item = FrameView::binary(buffer.to_vec());

        let _ = self
            .socket
            .send(item)
            .await
            .map_err(|_| ZConnectionError::CouldNotWrite);

        Ok(buffer.len())
    }

    async fn write_all(&mut self, buffer: &[u8]) -> ZResult<(), ZConnectionError> {
        #[cfg(all(feature = "async_wsocket", not(feature = "yawc")))]
        let item = Message::Binary(buffer.to_vec());
        #[cfg(feature = "yawc")]
        let item = FrameView::binary(buffer.to_vec());

        self.socket
            .send(item)
            .await
            .map_err(|_| ZConnectionError::CouldNotWrite)
    }

    async fn read(&mut self, buffer: &mut [u8]) -> ZResult<usize, ZConnectionError> {
        let Some(Ok(frame)) = self.socket.next().await else {
            return Err(ZConnectionError::CouldNotRead);
        };

        #[cfg(all(feature = "async_wsocket", not(feature = "yawc")))]
        match frame {
            Message::Binary(payload) => {
                let len = payload.len().min(buffer.len());
                buffer[..len].copy_from_slice(&payload[..len]);
                Ok(len)
            }
            _ => zbail!(ZConnectionError::CouldNotRead),
        }

        #[cfg(feature = "yawc")]
        match frame.opcode {
            OpCode::Binary => {
                let len = frame.payload.len().min(buffer.len());
                buffer[..len].copy_from_slice(&frame.payload[..len]);
                Ok(len)
            }
            _ => zbail!(ZConnectionError::CouldNotRead),
        }
    }

    async fn read_exact(&mut self, buffer: &mut [u8]) -> ZResult<(), ZConnectionError> {
        let Some(Ok(frame)) = self.socket.next().await else {
            return Err(ZConnectionError::CouldNotRead);
        };

        #[cfg(all(feature = "async_wsocket", not(feature = "yawc")))]
        match frame {
            Message::Binary(payload) if payload.len() == buffer.len() => {
                buffer.copy_from_slice(&payload);
                Ok(())
            }
            _ => zbail!(ZConnectionError::CouldNotRead),
        }

        #[cfg(feature = "yawc")]
        match (frame.opcode, frame.payload.len()) {
            (OpCode::Binary, len) if len == buffer.len() => {
                buffer.copy_from_slice(&frame.payload);
                Ok(())
            }
            _ => zbail!(ZConnectionError::CouldNotRead),
        }
    }
}

impl AbstractedWsTx for WasmWsTx<'_> {
    async fn write(&mut self, buffer: &[u8]) -> ZResult<usize, ZConnectionError> {
        #[cfg(all(feature = "async_wsocket", not(feature = "yawc")))]
        let item = Message::Binary(buffer.to_vec());
        #[cfg(feature = "yawc")]
        let item = FrameView::binary(buffer.to_vec());

        let _ = self
            .socket
            .send(item)
            .await
            .map_err(|_| ZConnectionError::CouldNotWrite);

        Ok(buffer.len())
    }

    async fn write_all(&mut self, buffer: &[u8]) -> ZResult<(), ZConnectionError> {
        #[cfg(all(feature = "async_wsocket", not(feature = "yawc")))]
        let item = Message::Binary(buffer.to_vec());
        #[cfg(feature = "yawc")]
        let item = FrameView::binary(buffer.to_vec());

        self.socket
            .send(item)
            .await
            .map_err(|_| ZConnectionError::CouldNotWrite)
    }
}

impl AbstractedWsRx for WasmWsRx<'_> {
    async fn read(&mut self, buffer: &mut [u8]) -> ZResult<usize, ZConnectionError> {
        let Some(Ok(frame)) = self.socket.next().await else {
            return Err(ZConnectionError::CouldNotRead);
        };

        #[cfg(all(feature = "async_wsocket", not(feature = "yawc")))]
        match frame {
            Message::Binary(payload) => {
                let len = payload.len().min(buffer.len());
                buffer[..len].copy_from_slice(&payload[..len]);
                Ok(len)
            }
            _ => zbail!(ZConnectionError::CouldNotRead),
        }

        #[cfg(feature = "yawc")]
        match frame.opcode {
            OpCode::Binary => {
                let len = frame.payload.len().min(buffer.len());
                buffer[..len].copy_from_slice(&frame.payload[..len]);
                Ok(len)
            }
            _ => zbail!(ZConnectionError::CouldNotRead),
        }
    }

    async fn read_exact(&mut self, buffer: &mut [u8]) -> ZResult<(), ZConnectionError> {
        let Some(Ok(frame)) = self.socket.next().await else {
            return Err(ZConnectionError::CouldNotRead);
        };

        #[cfg(all(feature = "async_wsocket", not(feature = "yawc")))]
        match frame {
            Message::Binary(payload) if payload.len() == buffer.len() => {
                buffer.copy_from_slice(&payload);
                Ok(())
            }
            _ => zbail!(ZConnectionError::CouldNotRead),
        }

        #[cfg(feature = "yawc")]
        match (frame.opcode, frame.payload.len()) {
            (OpCode::Binary, len) if len == buffer.len() => {
                buffer.copy_from_slice(&frame.payload);
                Ok(())
            }
            _ => zbail!(ZConnectionError::CouldNotRead),
        }
    }
}
