use zenoh_sansio::Transport;

use crate::{Link, ZLinkManager};

pub struct TransportLink<'a, LinkManager, Buff>
where
    LinkManager: ZLinkManager,
{
    link: Link<'a, LinkManager>,
    transport: Transport<Buff>,
}

pub struct TransportLinkManager<LinkManager> {
    link_manager: LinkManager,
}

impl<LinkManager> TransportLinkManager<LinkManager> {
    pub fn new(link_manager: LinkManager) -> Self {
        Self { link_manager }
    }
}
