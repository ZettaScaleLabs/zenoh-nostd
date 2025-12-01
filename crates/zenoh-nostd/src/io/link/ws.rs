use crate::{
    io::link::{ZLink, ZLinkInfo, ZLinkRx, ZLinkTx},
    platform::ws::{ZWsRx, ZWsStream, ZWsTx},
};

super::macros::define_link!(LinkWs, ZWsStream, stream, true, both);
super::macros::define_link!(LinkWsTx, ZWsTx, tx, true, tx);
super::macros::define_link!(LinkWsRx, ZWsRx, rx, true, rx);

impl<T: ZWsStream> LinkWs<T> {
    pub(crate) fn new(stream: T) -> Self {
        let mtu = stream.mtu();

        Self { stream, mtu }
    }
}

impl<T: ZWsStream> ZLink for LinkWs<T> {
    type Tx<'a>
        = LinkWsTx<T::Tx<'a>>
    where
        Self: 'a;

    type Rx<'a>
        = LinkWsRx<T::Rx<'a>>
    where
        Self: 'a;

    fn split(&mut self) -> (LinkWsTx<T::Tx<'_>>, LinkWsRx<T::Rx<'_>>) {
        let (tx, rx) = self.stream.split();
        let tx = LinkWsTx { tx, mtu: self.mtu };
        let rx = LinkWsRx { rx, mtu: self.mtu };
        (tx, rx)
    }
}
