use {
    async_net::TcpStream,
    std::net::SocketAddr,
    wtx::{
        collection::Vector,
        rng::Xorshift64,
        sync::Arc,
        web_socket::{
            Frame, OpCode, WebSocket, WebSocketBuffer, WebSocketPartsOwned, WebSocketPayloadOrigin,
            WebSocketReaderOwned, WebSocketReplier, WebSocketWriterOwned,
        },
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

pub struct StdWsStream {
    pub peer_addr: SocketAddr,
    pub stream: WebSocketReaderOwned<(), Xorshift64, TcpStream, true>,
    pub sink: WebSocketWriterOwned<(), Xorshift64, TcpStream, true>,
    pub replier: Arc<WebSocketReplier<true>>,
    pub read_buffer: Vector<u8>,
    pub write_buffer: Vector<u8>,
    pub mtu: u16,
}

impl StdWsStream {
    pub fn new(
        peer_addr: SocketAddr,
        stream: WebSocket<(), Xorshift64, TcpStream, WebSocketBuffer, true>,
    ) -> Self {
        let WebSocketPartsOwned {
            reader,
            replier,
            writer,
        } = stream
            .into_parts(|s| (s.clone(), s))
            .expect("Failed to split WebSocket");
        Self {
            peer_addr,
            stream: reader,
            sink: writer,
            replier,
            read_buffer: Vector::<u8>::new(),
            write_buffer: Vector::<u8>::new(),
            mtu: u16::MAX,
        }
    }
}

pub struct StdWsTx<'a> {
    pub sink: &'a mut WebSocketWriterOwned<(), Xorshift64, TcpStream, true>,
    pub replier: &'a WebSocketReplier<true>,
    pub write_buffer: &'a mut Vector<u8>,
}

pub struct StdWsRx<'a> {
    pub stream: &'a mut WebSocketReaderOwned<(), Xorshift64, TcpStream, true>,
    pub read_buffer: &'a mut Vector<u8>,
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
            replier: &self.replier,
            write_buffer: &mut self.write_buffer,
        };
        let rx = StdWsRx {
            stream: &mut self.stream,
            read_buffer: &mut self.read_buffer,
        };
        (tx, rx)
    }

    async fn write(&mut self, buffer: &[u8]) -> ZResult<usize, ZConnectionError> {
        let mut tx = StdWsTx {
            sink: &mut self.sink,
            replier: &self.replier,
            write_buffer: &mut self.write_buffer,
        };
        tx.write(buffer).await
    }

    async fn write_all(&mut self, buffer: &[u8]) -> ZResult<(), ZConnectionError> {
        self.write(buffer).await.map(|_| ())
    }

    async fn read(&mut self, buffer: &mut [u8]) -> ZResult<usize, ZConnectionError> {
        let mut rx = StdWsRx {
            stream: &mut self.stream,
            read_buffer: &mut self.read_buffer,
        };
        rx.read(buffer).await
    }

    async fn read_exact(&mut self, buffer: &mut [u8]) -> ZResult<(), ZConnectionError> {
        self.read(buffer).await.map(|_| ())
    }
}

impl AbstractedWsTx for StdWsTx<'_> {
    async fn write(&mut self, buffer: &[u8]) -> ZResult<usize, ZConnectionError> {
        self.write_buffer.clear();
        self.write_buffer
            .extend_from_copyable_slice(buffer)
            .map_err(|_| {
                zenoh_nostd::error!("Failed to extend write buffer");
                ZConnectionError::CouldNotWrite
            })?;
        let payload = self.write_buffer.as_slice_mut();
        self.sink
            .write_frame(&mut Frame::new_fin(OpCode::Binary, payload))
            .await
            .map_err(|_| {
                zenoh_nostd::error!("Could not write frame");
                ZConnectionError::CouldNotWrite
            })
            .map(|_| buffer.len())
    }

    async fn write_all(&mut self, buffer: &[u8]) -> ZResult<(), ZConnectionError> {
        self.write(buffer).await.map(|_| ())
    }
}

impl AbstractedWsRx for StdWsRx<'_> {
    async fn read(&mut self, buffer: &mut [u8]) -> ZResult<usize, ZConnectionError> {
        self.read_buffer.clear();
        let Ok(frame) = self
            .stream
            .read_frame(self.read_buffer, WebSocketPayloadOrigin::Consistent)
            .await
        else {
            zenoh_nostd::error!("Could not read frame");
            return Err(ZConnectionError::CouldNotRead);
        };
        match frame.op_code() {
            OpCode::Binary => {
                let len = frame.payload().len().min(buffer.len());
                buffer[..len].copy_from_slice(&frame.payload()[..len]);
                Ok(len)
            }
            _ => {
                zenoh_nostd::error!("Could not read frame into buffer");
                zbail!(ZConnectionError::CouldNotRead);
            }
        }
    }

    async fn read_exact(&mut self, buffer: &mut [u8]) -> ZResult<(), ZConnectionError> {
        self.read(buffer).await.map(|_| ())
    }
}
