use zenoh_sansio::{TransportRx, ZTransportRx};

use super::{LinkRx, ZLinkManager, ZLinkRx, ZTransportLinkRx};

pub struct TransportLinkRx<'res, 'transport, LinkManager, Buff>
where
    LinkManager: ZLinkManager,
{
    link: LinkRx<'res, 'transport, LinkManager>,
    transport: &'transport mut TransportRx<Buff>,
}

impl<'res, 'transport, LinkManager, Buff> TransportLinkRx<'res, 'transport, LinkManager, Buff>
where
    LinkManager: ZLinkManager,
{
    pub fn new(
        link: LinkRx<'res, 'transport, LinkManager>,
        transport: &'transport mut TransportRx<Buff>,
    ) -> Self {
        Self { link, transport }
    }

    pub fn transport(&self) -> &TransportRx<Buff> {
        &self.transport
    }

    pub fn transport_mut(&mut self) -> &mut TransportRx<Buff> {
        &mut self.transport
    }
}

impl<'res, 'transport, LinkManager, Buff> ZTransportLinkRx
    for TransportLinkRx<'res, 'transport, LinkManager, Buff>
where
    LinkManager: ZLinkManager,
    Buff: AsMut<[u8]> + AsRef<[u8]>,
{
    fn rx(&mut self) -> (&mut impl ZLinkRx, &mut impl ZTransportRx) {
        (&mut self.link, self.transport)
    }
}
