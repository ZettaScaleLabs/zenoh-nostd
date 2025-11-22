use {
    // futures_lite::StreamExt as _,
    futures_util::{
        SinkExt as _, StreamExt as _,
        stream::{SplitSink, SplitStream},
    },
    std::net::SocketAddr,
    yawc::{FrameView, OpCode, WebSocket},
    zenoh_nostd::{
        ZResult,
        platform::{
            ZConnectionError,
            ws::{AbstractedWsRx, AbstractedWsStream, AbstractedWsTx},
        },
        zbail,
    },
};

pub struct StdWsStream {
    pub peer_addr: SocketAddr,
    pub sink: SplitSink<WebSocket, FrameView>,
    pub stream: SplitStream<WebSocket>,
    pub mtu: u16,
}

impl StdWsStream {
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

pub struct StdWsTx<'a> {
    pub sink: &'a mut SplitSink<WebSocket, FrameView>,
}

pub struct StdWsRx<'a> {
    pub stream: &'a mut SplitStream<WebSocket>,
}

impl AbstractedWsStream for StdWsStream {
    type Tx<'a> = StdWsTx<'a>;
    type Rx<'a> = StdWsRx<'a>;

    fn mtu(&self) -> u16 {
        self.mtu
    }

    fn split(&mut self) -> (Self::Tx<'_>, Self::Rx<'_>) {
        let tx = StdWsTx {
            sink: &mut self.sink,
        };
        let rx = StdWsRx {
            stream: &mut self.stream,
        };
        (tx, rx)
    }

    async fn write(&mut self, buffer: &[u8]) -> ZResult<usize, ZConnectionError> {
        self.sink
            .send(FrameView::binary(buffer.to_vec()))
            .await
            .map_err(|_| ZConnectionError::CouldNotWrite)
            .map(|_| buffer.len())
    }

    async fn write_all(&mut self, buffer: &[u8]) -> ZResult<(), ZConnectionError> {
        self.sink
            .send(FrameView::binary(buffer.to_vec()))
            .await
            .map_err(|_| ZConnectionError::CouldNotWrite)
    }

    async fn read(&mut self, buffer: &mut [u8]) -> ZResult<usize, ZConnectionError> {
        let Some(frame) = self.stream.next().await else {
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
        let Some(frame) = self.stream.next().await else {
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

impl AbstractedWsTx for StdWsTx<'_> {
    async fn write(&mut self, buffer: &[u8]) -> ZResult<usize, ZConnectionError> {
        self.sink
            .send(FrameView::binary(buffer.to_vec()))
            .await
            .map_err(|_| ZConnectionError::CouldNotWrite)
            .map(|_| buffer.len())
    }

    async fn write_all(&mut self, buffer: &[u8]) -> ZResult<(), ZConnectionError> {
        self.sink
            .send(FrameView::binary(buffer.to_vec()))
            .await
            .map_err(|_| ZConnectionError::CouldNotWrite)
    }
}

impl AbstractedWsRx for StdWsRx<'_> {
    async fn read(&mut self, buffer: &mut [u8]) -> ZResult<usize, ZConnectionError> {
        let Some(frame) = self.stream.next().await else {
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
        let Some(frame) = self.stream.next().await else {
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
