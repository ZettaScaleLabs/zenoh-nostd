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
    pub buffer: Vector<u8>,
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
            buffer: Vector::<u8>::new(),
            mtu: u16::MAX,
        }
    }
}

pub struct StdWsTx<'a> {
    pub sink: &'a mut WebSocketWriterOwned<(), Xorshift64, TcpStream, true>,
    pub replier: Arc<WebSocketReplier<true>>,
}

pub struct StdWsRx<'a> {
    pub stream: &'a mut WebSocketReaderOwned<(), Xorshift64, TcpStream, true>,
    pub buffer: &'a mut Vector<u8>,
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
            replier: self.replier.clone(),
        };
        let rx = StdWsRx {
            stream: &mut self.stream,
            buffer: &mut self.buffer,
        };
        (tx, rx)
    }

    async fn write(&mut self, buffer: &[u8]) -> ZResult<usize, ZConnectionError> {
        self.sink
            .write_frame(&mut Frame::new_fin(
                OpCode::Binary,
                buffer.to_vec().as_mut_slice(),
            ))
            .await
            .map_err(|_| {
                zenoh_nostd::error!("Could not read write frame");
                ZConnectionError::CouldNotWrite
            })
            .map(|_| buffer.len())
    }

    async fn write_all(&mut self, buffer: &[u8]) -> ZResult<(), ZConnectionError> {
        self.sink
            .write_frame(&mut Frame::new_fin(
                OpCode::Binary,
                buffer.to_vec().as_mut_slice(),
            ))
            .await
            .map_err(|_| {
                zenoh_nostd::error!("Could not read write all frame");
                ZConnectionError::CouldNotWrite
            })
    }

    async fn read(&mut self, buffer: &mut [u8]) -> ZResult<usize, ZConnectionError> {
        let Ok(frame) = self
            .stream
            .read_frame(&mut self.buffer, WebSocketPayloadOrigin::Adaptive)
            .await
        else {
            zenoh_nostd::error!("Could not read frame");
            return Err(ZConnectionError::CouldNotRead);
        };
        match frame.op_code() {
            OpCode::Binary => {
                let len = frame.payload().len().min(buffer.len());
                buffer[..len].copy_from_slice(&frame.payload()[..len]);
                self.buffer.clear();
                Ok(len)
            }
            _ => {
                zenoh_nostd::error!("Could not read frame into buffer");
                zbail!(ZConnectionError::CouldNotRead);
            }
        }
    }

    async fn read_exact(&mut self, buffer: &mut [u8]) -> ZResult<(), ZConnectionError> {
        let Ok(frame) = self
            .stream
            .read_frame(&mut self.buffer, WebSocketPayloadOrigin::Adaptive)
            .await
        else {
            return Err(ZConnectionError::CouldNotRead);
        };
        match (frame.op_code(), frame.payload().len()) {
            (OpCode::Binary, len) if len == buffer.len() => {
                buffer.copy_from_slice(frame.payload());
                self.buffer.clear();
                Ok(())
            }
            _ => {
                zenoh_nostd::error!("Could not read exact frame into buffer");
                zbail!(ZConnectionError::CouldNotRead);
            }
        }
    }
}

impl AbstractedWsTx for StdWsTx<'_> {
    async fn write(&mut self, buffer: &[u8]) -> ZResult<usize, ZConnectionError> {
        self.sink
            .write_frame(&mut Frame::new_fin(
                OpCode::Binary,
                buffer.to_vec().as_mut_slice(),
            ))
            .await
            .map_err(|_| {
                zenoh_nostd::error!("Could not read write frame");
                ZConnectionError::CouldNotWrite
            })
            .map(|_| buffer.len())
    }

    async fn write_all(&mut self, buffer: &[u8]) -> ZResult<(), ZConnectionError> {
        self.sink
            .write_frame(&mut Frame::new_fin(
                OpCode::Binary,
                buffer.to_vec().as_mut_slice(),
            ))
            .await
            .map_err(|_| {
                zenoh_nostd::error!("Could not read write all frame");
                ZConnectionError::CouldNotWrite
            })
    }
}

impl AbstractedWsRx for StdWsRx<'_> {
    async fn read(&mut self, buffer: &mut [u8]) -> ZResult<usize, ZConnectionError> {
        let Ok(frame) = self
            .stream
            .read_frame(self.buffer, WebSocketPayloadOrigin::Adaptive)
            .await
        else {
            zenoh_nostd::error!("Could not read frame");
            return Err(ZConnectionError::CouldNotRead);
        };
        match frame.op_code() {
            OpCode::Binary => {
                let len = frame.payload().len().min(buffer.len());
                buffer[..len].copy_from_slice(&frame.payload()[..len]);
                self.buffer.clear();
                Ok(len)
            }
            _ => {
                zenoh_nostd::error!("Could not read frame into buffer");
                zbail!(ZConnectionError::CouldNotRead);
            }
        }
    }

    async fn read_exact(&mut self, buffer: &mut [u8]) -> ZResult<(), ZConnectionError> {
        let Ok(frame) = self
            .stream
            .read_frame(self.buffer, WebSocketPayloadOrigin::Adaptive)
            .await
        else {
            zenoh_nostd::error!("Could not read frame");
            return Err(ZConnectionError::CouldNotRead);
        };
        match (frame.op_code(), frame.payload().len()) {
            (OpCode::Binary, len) if len == buffer.len() => {
                buffer.copy_from_slice(frame.payload());
                self.buffer.clear();
                Ok(())
            }
            _ => {
                zenoh_nostd::error!("Could not read exact frame into buffer");
                zbail!(ZConnectionError::CouldNotRead);
            }
        }
    }
}
