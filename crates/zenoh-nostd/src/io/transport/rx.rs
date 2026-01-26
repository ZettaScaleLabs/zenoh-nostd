use zenoh_sansio::{TransportRx, ZTransportRx};

use super::{LinkRx, ZLinkManager, ZLinkRx, ZTransportLinkRx};

pub struct TransportLinkRx<'ext, 'transport, LinkManager, Buff>
where
    LinkManager: ZLinkManager,
{
    link: LinkRx<'ext, 'transport, LinkManager>,
    transport: &'transport mut TransportRx<Buff>,
}

impl<'ext, 'transport, LinkManager, Buff> TransportLinkRx<'ext, 'transport, LinkManager, Buff>
where
    LinkManager: ZLinkManager,
{
    pub fn new(
        link: LinkRx<'ext, 'transport, LinkManager>,
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

impl<'ext, 'transport, LinkManager, Buff> ZTransportLinkRx
    for TransportLinkRx<'ext, 'transport, LinkManager, Buff>
where
    LinkManager: ZLinkManager,
    Buff: AsMut<[u8]> + AsRef<[u8]>,
{
    fn rx(&mut self) -> (&mut impl ZLinkRx, &mut impl ZTransportRx) {
        (&mut self.link, self.transport)
    }
}
