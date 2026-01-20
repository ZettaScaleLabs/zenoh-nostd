use crate::{ZLink, ZLinkInfo, ZLinkRx, ZLinkTx};

impl ZLinkInfo for () {
    fn mtu(&self) -> u16 {
        unimplemented!()
    }

    fn is_streamed(&self) -> bool {
        unimplemented!()
    }
}

impl ZLinkTx for () {
    fn write_all(
        &mut self,
        _: &[u8],
    ) -> impl Future<Output = core::result::Result<(), zenoh_proto::LinkError>> {
        async { unimplemented!() }
    }
}

impl ZLinkRx for () {
    fn read(
        &mut self,
        _: &mut [u8],
    ) -> impl Future<Output = core::result::Result<usize, zenoh_proto::LinkError>> {
        async { unimplemented!() }
    }

    fn read_exact(
        &mut self,
        _: &mut [u8],
    ) -> impl Future<Output = core::result::Result<(), zenoh_proto::LinkError>> {
        async { unimplemented!() }
    }
}

impl ZLink for () {
    type Rx<'a>
        = ()
    where
        Self: 'a;
    type Tx<'a>
        = ()
    where
        Self: 'a;

    fn split(&mut self) -> (Self::Tx<'_>, Self::Rx<'_>) {
        unimplemented!()
    }
}
