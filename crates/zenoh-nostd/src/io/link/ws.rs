use {
    crate::{
        io::ZLinkError,
        platform::ws::{AbstractedWsRx, AbstractedWsStream, AbstractedWsTx},
    },
    zenoh_proto::ZResult,
};

pub(crate) struct LinkWs<T: AbstractedWsStream> {
    stream: T,
    mtu: u16,
}

pub(crate) struct LinkWsTx<T: AbstractedWsTx> {
    tx: T,
}

pub(crate) struct LinkWsRx<T: AbstractedWsRx> {
    rx: T,
}

impl<T: AbstractedWsStream> LinkWs<T> {
    pub(crate) fn new(stream: T) -> Self {
        let mtu = stream.mtu();

        Self { stream, mtu }
    }

    pub(crate) fn split(&mut self) -> (LinkWsTx<T::Tx<'_>>, LinkWsRx<T::Rx<'_>>) {
        let (tx, rx) = self.stream.split();
        let tx = LinkWsTx { tx };
        let rx = LinkWsRx { rx };
        (tx, rx)
    }

    pub(crate) fn mtu(&self) -> u16 {
        self.mtu
    }

    pub(crate) fn is_streamed(&self) -> bool {
        true
    }

    pub(crate) async fn write(&mut self, buffer: &[u8]) -> ZResult<usize, ZLinkError> {
        self.stream.write(buffer).await.map_err(|e| e.into())
    }

    pub(crate) async fn write_all(&mut self, buffer: &[u8]) -> ZResult<(), ZLinkError> {
        self.stream.write_all(buffer).await.map_err(|e| e.into())
    }

    pub(crate) async fn read(&mut self, buffer: &mut [u8]) -> ZResult<usize, ZLinkError> {
        self.stream.read(buffer).await.map_err(|e| e.into())
    }

    pub(crate) async fn read_exact(&mut self, buffer: &mut [u8]) -> ZResult<(), ZLinkError> {
        self.stream.read_exact(buffer).await.map_err(|e| e.into())
    }
}

impl<T: AbstractedWsTx> LinkWsTx<T> {
    pub(crate) fn is_streamed(&self) -> bool {
        true
    }

    pub(crate) async fn write(&mut self, buffer: &[u8]) -> ZResult<usize, ZLinkError> {
        self.tx.write(buffer).await.map_err(|e| e.into())
    }

    pub(crate) async fn write_all(&mut self, buffer: &[u8]) -> ZResult<(), ZLinkError> {
        self.tx.write_all(buffer).await.map_err(|e| e.into())
    }
}

impl<T: AbstractedWsRx> LinkWsRx<T> {
    pub(crate) fn is_streamed(&self) -> bool {
        true
    }

    pub(crate) async fn read(&mut self, buffer: &mut [u8]) -> ZResult<usize, ZLinkError> {
        self.rx.read(buffer).await.map_err(|e| e.into())
    }

    pub(crate) async fn read_exact(&mut self, buffer: &mut [u8]) -> ZResult<(), ZLinkError> {
        self.rx.read_exact(buffer).await.map_err(|e| e.into())
    }
}
