use {
    core::net::SocketAddr,
    futures_util::{
        SinkExt as _, StreamExt as _,
        stream::{SplitSink, SplitStream},
    },
    yawc::{
        WebSocket,
        frame::{FrameView, OpCode},
    },
    zenoh_nostd::{
        ZResult,
        platform::{
            ZConnectionError,
            ws::{AbstractedWsRx, AbstractedWsStream, AbstractedWsTx},
        },
        zbail,
    },
};

pub struct WasmWsStream {
    pub peer_addr: SocketAddr,
    pub sink: SplitSink<WebSocket, FrameView>,
    pub stream: SplitStream<WebSocket>,
    pub mtu: u16,
}

impl WasmWsStream {
    pub fn new(peer_addr: SocketAddr, stream: WebSocket) -> Self {
        let (sink, stream) = stream.split();
        Self {
            peer_addr,
            sink,
            stream,
            mtu: u16::MAX,
        }
    }
}

pub struct WasmWsTx<'a> {
    pub sink: &'a mut SplitSink<WebSocket, FrameView>,
}

pub struct WasmWsRx<'a> {
    pub stream: &'a mut SplitStream<WebSocket>,
}

impl AbstractedWsStream for WasmWsStream {
    type Tx<'a> = WasmWsTx<'a>;
    type Rx<'a> = WasmWsRx<'a>;

    fn mtu(&self) -> u16 {
        self.mtu
    }

    fn split(&mut self) -> (Self::Tx<'_>, Self::Rx<'_>) {
        let tx = WasmWsTx {
            sink: &mut self.sink,
        };
        let rx = WasmWsRx {
            stream: &mut self.stream,
        };
        (tx, rx)
    }

    async fn write(&mut self, buffer: &[u8]) -> ZResult<usize, ZConnectionError> {
        let mut tx = WasmWsTx {
            sink: &mut self.sink,
        };
        tx.write(buffer).await
    }

    async fn write_all(&mut self, buffer: &[u8]) -> ZResult<(), ZConnectionError> {
        self.write(buffer).await.map(|_| ())
    }

    async fn read(&mut self, buffer: &mut [u8]) -> ZResult<usize, ZConnectionError> {
        let mut rx = WasmWsRx {
            stream: &mut self.stream,
        };
        rx.read(buffer).await
    }

    async fn read_exact(&mut self, buffer: &mut [u8]) -> ZResult<(), ZConnectionError> {
        let mut rx = WasmWsRx {
            stream: &mut self.stream,
        };
        rx.read_exact(buffer).await
    }
}

impl AbstractedWsTx for WasmWsTx<'_> {
    async fn write(&mut self, buffer: &[u8]) -> ZResult<usize, ZConnectionError> {
        let item = FrameView::binary(buffer.to_vec());
        self.sink
            .send(item)
            .await
            .map_err(|_| ZConnectionError::CouldNotWrite)
            .map(|_| buffer.len())
    }

    async fn write_all(&mut self, buffer: &[u8]) -> ZResult<(), ZConnectionError> {
        self.write(buffer).await.map(|_| ())
    }
}

impl AbstractedWsRx for WasmWsRx<'_> {
    async fn read(&mut self, buffer: &mut [u8]) -> ZResult<usize, ZConnectionError> {
        let Some(Ok(frame)) = self.stream.next().await else {
            return Err(ZConnectionError::CouldNotRead);
        };
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
        let Some(Ok(frame)) = self.stream.next().await else {
            return Err(ZConnectionError::CouldNotRead);
        };
        match (frame.opcode, frame.payload.len()) {
            (OpCode::Binary, len) if len == buffer.len() => {
                buffer.copy_from_slice(&frame.payload);
                Ok(())
            }
            _ => zbail!(ZConnectionError::CouldNotRead),
        }
    }
}
