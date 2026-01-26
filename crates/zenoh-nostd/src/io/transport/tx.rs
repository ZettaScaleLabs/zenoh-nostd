use zenoh_sansio::{TransportTx, ZTransportTx};

use super::{LinkTx, ZLinkManager, ZLinkTx, ZTransportLinkTx};

pub struct TransportLinkTx<'ext, 'transport, LinkManager, Buff>
where
    LinkManager: ZLinkManager,
{
    link: LinkTx<'ext, 'transport, LinkManager>,
    transport: &'transport mut TransportTx<Buff>,
}

impl<'ext, 'transport, LinkManager, Buff> TransportLinkTx<'ext, 'transport, LinkManager, Buff>
where
    LinkManager: ZLinkManager,
{
    pub fn new(
        link: LinkTx<'ext, 'transport, LinkManager>,
        transport: &'transport mut TransportTx<Buff>,
    ) -> Self {
        Self { link, transport }
    }

    pub fn transport(&self) -> &TransportTx<Buff> {
        &self.transport
    }

    pub fn transport_mut(&mut self) -> &mut TransportTx<Buff> {
        &mut self.transport
    }
}

impl<'ext, 'transport, LinkManager, Buff> ZTransportLinkTx
    for TransportLinkTx<'ext, 'transport, LinkManager, Buff>
where
    LinkManager: ZLinkManager,
    Buff: AsMut<[u8]> + AsRef<[u8]>,
{
    fn tx(&mut self) -> (&mut impl ZLinkTx, &mut impl ZTransportTx) {
        (&mut self.link, self.transport)
    }
}
