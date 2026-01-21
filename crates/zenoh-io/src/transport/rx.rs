use zenoh_sansio::{TransportRx, ZTransportRx};

use crate::{LinkRx, ZLinkManager, ZLinkRx, ZTransportLinkRx};

pub struct TransportLinkRx<'p, 'a, LinkManager, Buff>
where
    LinkManager: ZLinkManager,
{
    link: LinkRx<'p, 'a, LinkManager>,
    transport: &'p mut TransportRx<Buff>,
}

impl<'p, 'a, LinkManager, Buff> TransportLinkRx<'p, 'a, LinkManager, Buff>
where
    LinkManager: ZLinkManager,
{
    pub fn new(link: LinkRx<'p, 'a, LinkManager>, transport: &'p mut TransportRx<Buff>) -> Self {
        Self { link, transport }
    }
}

impl<'p, 'a, LinkManager, Buff> ZTransportLinkRx for TransportLinkRx<'p, 'a, LinkManager, Buff>
where
    LinkManager: ZLinkManager,
    Buff: AsMut<[u8]> + AsRef<[u8]>,
{
    fn rx(&mut self) -> (&mut impl ZLinkRx, &mut impl ZTransportRx) {
        (&mut self.link, self.transport)
    }
}
