use zenoh_sansio::{TransportTx, ZTransportTx};

use crate::{LinkTx, ZLinkManager, ZLinkTx, ZTransportLinkTx};

pub struct TransportLinkTx<'p, 'a, LinkManager, Buff>
where
    LinkManager: ZLinkManager,
{
    link: LinkTx<'p, 'a, LinkManager>,
    transport: &'p mut TransportTx<Buff>,
}

impl<'p, 'a, LinkManager, Buff> TransportLinkTx<'p, 'a, LinkManager, Buff>
where
    LinkManager: ZLinkManager,
{
    pub fn new(link: LinkTx<'p, 'a, LinkManager>, transport: &'p mut TransportTx<Buff>) -> Self {
        Self { link, transport }
    }
}

impl<'p, 'a, LinkManager, Buff> ZTransportLinkTx for TransportLinkTx<'p, 'a, LinkManager, Buff>
where
    LinkManager: ZLinkManager,
    Buff: AsMut<[u8]> + AsRef<[u8]>,
{
    fn tx(&mut self) -> (&mut impl ZLinkTx, &mut impl ZTransportTx) {
        (&mut self.link, self.transport)
    }
}
