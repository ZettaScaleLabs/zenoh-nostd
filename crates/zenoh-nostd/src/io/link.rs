use zenoh_proto::{Endpoint, LinkError};

pub trait ZLinkInfo {
    fn mtu(&self) -> u16;
    fn is_streamed(&self) -> bool;
}

pub trait ZLinkTx: ZLinkInfo {
    fn write_all(
        &mut self,
        buffer: &[u8],
    ) -> impl Future<Output = core::result::Result<(), zenoh_proto::LinkError>>;
}

pub trait ZLinkRx: ZLinkInfo {
    fn read(
        &mut self,
        buffer: &mut [u8],
    ) -> impl Future<Output = core::result::Result<usize, zenoh_proto::LinkError>>;

    fn read_exact(
        &mut self,
        buffer: &mut [u8],
    ) -> impl Future<Output = core::result::Result<(), zenoh_proto::LinkError>>;
}

pub trait ZLink: ZLinkInfo + ZLinkTx + ZLinkRx {
    type Tx<'link>: ZLinkTx + ZLinkInfo
    where
        Self: 'link;

    type Rx<'link>: ZLinkRx + ZLinkInfo
    where
        Self: 'link;

    fn split(&mut self) -> (Self::Tx<'_>, Self::Rx<'_>);
}

pub trait ZLinkManager {
    type Link<'a>: ZLink
    where
        Self: 'a;

    fn connect(
        &self,
        endpoint: Endpoint<'_>,
    ) -> impl Future<Output = core::result::Result<Self::Link<'_>, LinkError>>;

    fn listen(
        &self,
        endpoint: Endpoint<'_>,
    ) -> impl Future<Output = core::result::Result<Self::Link<'_>, LinkError>>;
}
