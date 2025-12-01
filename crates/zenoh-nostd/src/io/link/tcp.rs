use crate::{
    io::link::{ZLink, ZLinkInfo, ZLinkRx, ZLinkTx},
    platform::tcp::{ZTcpRx, ZTcpStream, ZTcpTx},
};

super::macros::define_link!(LinkTcp, ZTcpStream, stream, true, both);
super::macros::define_link!(LinkTcpTx, ZTcpTx, tx, true, tx);
super::macros::define_link!(LinkTcpRx, ZTcpRx, rx, true, rx);

impl<T: ZTcpStream> LinkTcp<T> {
    pub(crate) fn new(stream: T) -> Self {
        let mtu = stream.mtu();

        Self { stream, mtu }
    }
}

impl<T: ZTcpStream> ZLink for LinkTcp<T> {
    type Tx<'a>
        = LinkTcpTx<T::Tx<'a>>
    where
        Self: 'a;

    type Rx<'a>
        = LinkTcpRx<T::Rx<'a>>
    where
        Self: 'a;

    fn split(&mut self) -> (LinkTcpTx<T::Tx<'_>>, LinkTcpRx<T::Rx<'_>>) {
        let (tx, rx) = self.stream.split();
        let tx = LinkTcpTx { tx, mtu: self.mtu };
        let rx = LinkTcpRx { rx, mtu: self.mtu };
        (tx, rx)
    }
}
