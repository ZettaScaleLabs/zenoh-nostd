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
        platform::ws::{ZWebSocket, ZWsRx, ZWsTx},
        zbail,
    },
};

pub struct WasmWebSocket {
    pub peer_addr: SocketAddr,
    pub sink: SplitSink<WebSocket, FrameView>,
    pub stream: SplitStream<WebSocket>,
    pub mtu: u16,
}

impl WasmWebSocket {
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

impl ZWebSocket for WasmWebSocket {
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
}

impl ZWsTx for WasmWebSocket {
    async fn write(
        &mut self,
        buffer: &[u8],
    ) -> core::result::Result<usize, zenoh_nostd::LinkError> {
        let mut tx = WasmWsTx {
            sink: &mut self.sink,
        };
        tx.write(buffer).await
    }

    async fn write_all(
        &mut self,
        buffer: &[u8],
    ) -> core::result::Result<(), zenoh_nostd::LinkError> {
        self.write(buffer).await.map(|_| ())
    }
}

impl ZWsTx for WasmWsTx<'_> {
    async fn write(
        &mut self,
        buffer: &[u8],
    ) -> core::result::Result<usize, zenoh_nostd::LinkError> {
        let item = FrameView::binary(buffer.to_vec());
        self.sink
            .send(item)
            .await
            .map_err(|_| zenoh_nostd::LinkError::LinkTxFailed)
            .map(|_| buffer.len())
    }

    async fn write_all(
        &mut self,
        buffer: &[u8],
    ) -> core::result::Result<(), zenoh_nostd::LinkError> {
        self.write(buffer).await.map(|_| ())
    }
}

impl ZWsRx for WasmWebSocket {
    async fn read(
        &mut self,
        buffer: &mut [u8],
    ) -> core::result::Result<usize, zenoh_nostd::LinkError> {
        let mut rx = WasmWsRx {
            stream: &mut self.stream,
        };
        rx.read(buffer).await
    }

    async fn read_exact(
        &mut self,
        buffer: &mut [u8],
    ) -> core::result::Result<(), zenoh_nostd::LinkError> {
        let mut rx = WasmWsRx {
            stream: &mut self.stream,
        };
        rx.read_exact(buffer).await
    }
}

impl ZWsRx for WasmWsRx<'_> {
    async fn read(
        &mut self,
        buffer: &mut [u8],
    ) -> core::result::Result<usize, zenoh_nostd::LinkError> {
        let Some(Ok(frame)) = self.stream.next().await else {
            return Err(zenoh_nostd::LinkError::LinkRxFailed);
        };
        match frame.opcode {
            OpCode::Binary => {
                let len = frame.payload.len().min(buffer.len());
                buffer[..len].copy_from_slice(&frame.payload[..len]);
                Ok(len)
            }
            _ => zbail!(zenoh_nostd::LinkError::LinkRxFailed),
        }
    }

    async fn read_exact(
        &mut self,
        buffer: &mut [u8],
    ) -> core::result::Result<(), zenoh_nostd::LinkError> {
        let Some(Ok(frame)) = self.stream.next().await else {
            return Err(zenoh_nostd::LinkError::LinkRxFailed);
        };
        match (frame.opcode, frame.payload.len()) {
            (OpCode::Binary, len) if len == buffer.len() => {
                buffer.copy_from_slice(&frame.payload);
                Ok(())
            }
            _ => zbail!(zenoh_nostd::LinkError::LinkRxFailed),
        }
    }
}
