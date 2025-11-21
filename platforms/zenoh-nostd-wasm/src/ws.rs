use {
    async_wsocket::{
        Error, Message, WebSocket,
        futures_util::{
            SinkExt, StreamExt,
            stream::{SplitSink, SplitStream},
        },
    },
    core::net::SocketAddr,
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

pub struct WasmWsTx<'a> {
    pub socket: SplitSink<WebSocket, Message>,
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
        let _ = self
            .socket
            .send(Message::Binary(buffer.to_vec()))
            .await
            .map_err(|_| ZConnectionError::CouldNotWrite);

        Ok(buffer.len())
    }

    async fn write_all(&mut self, buffer: &[u8]) -> ZResult<(), ZConnectionError> {
        self.write(buffer)
            .await
            .map_err(|_| ZConnectionError::CouldNotWrite)
            .map(|_| ())
    }

    async fn read(&mut self, buffer: &mut [u8]) -> ZResult<usize, ZConnectionError> {
        let msg: Option<Result<Message, ZConnectionError>> = self
            .socket
            .next()
            .await
            .map(|e: Result<Message, Error>| e.map_err(|_| ZConnectionError::CouldNotRead));

        if let Some(Ok(Message::Binary(data))) = msg {
            let len = data.len().min(buffer.len());
            buffer[..len].copy_from_slice(&data[..len]);
            Ok(len)
        } else {
            zbail!(ZConnectionError::CouldNotRead)
        }
    }

    async fn read_exact(&mut self, buffer: &mut [u8]) -> ZResult<(), ZConnectionError> {
        let msg = self
            .socket
            .next()
            .await
            .map(|e: Result<Message, Error>| e.map_err(|_| ZConnectionError::CouldNotRead));

        if let Some(Ok(Message::Binary(data))) = msg {
            if data.len() != buffer.len() {
                zbail!(ZConnectionError::CouldNotRead);
            }
            buffer.copy_from_slice(&data);
            Ok(())
        } else {
            zbail!(ZConnectionError::CouldNotRead)
        }
    }
}

impl AbstractedWsTx for WasmWsTx<'_> {
    async fn write(&mut self, buffer: &[u8]) -> ZResult<usize, ZConnectionError> {
        let _ = self
            .socket
            .send(Message::Binary(buffer.to_vec()))
            .await
            .map_err(|_| ZConnectionError::CouldNotWrite);

        Ok(buffer.len())
    }

    async fn write_all(&mut self, buffer: &[u8]) -> ZResult<(), ZConnectionError> {
        self.write(buffer)
            .await
            .map_err(|_| ZConnectionError::CouldNotWrite)
            .map(|_| ())
    }
}

impl AbstractedWsRx for WasmWsRx<'_> {
    async fn read(&mut self, buffer: &mut [u8]) -> ZResult<usize, ZConnectionError> {
        let msg: Option<Result<Message, ZConnectionError>> =
            self.socket.next().await.map(|e: Result<Message, Error>| {
                e.map_err(|_| {
                    zenoh_nostd::error!("Could not read from stream");
                    ZConnectionError::CouldNotRead
                })
            });

        if let Some(Ok(Message::Binary(data))) = msg {
            let len = data.len().min(buffer.len());
            buffer[..len].copy_from_slice(&data[..len]);
            Ok(len)
        } else {
            zenoh_nostd::error!("Could not read data");
            zbail!(ZConnectionError::CouldNotRead)
        }
    }

    async fn read_exact(&mut self, buffer: &mut [u8]) -> ZResult<(), ZConnectionError> {
        let msg = self.socket.next().await.map(|e: Result<Message, Error>| {
            e.map_err(|_| {
                zenoh_nostd::error!("Could not read from stream");
                ZConnectionError::CouldNotRead
            })
        });

        if let Some(Ok(Message::Binary(data))) = msg {
            if data.len() != buffer.len() {
                zenoh_nostd::error!("Could not read exact data");
                zbail!(ZConnectionError::CouldNotRead);
            }
            buffer.copy_from_slice(&data);
            Ok(())
        } else {
            zbail!(ZConnectionError::CouldNotRead)
        }
    }
}
