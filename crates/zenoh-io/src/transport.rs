use zenoh_proto::{EndPoint, TransportLinkError};
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

    pub async fn connect<Buff>(
        &self,
        endpoint: EndPoint<'_>,
        buff: Buff,
    ) -> core::result::Result<TransportLink<'_, LinkManager, Buff>, TransportLinkError>
    where
        LinkManager: ZLinkManager,
    {
        todo!()
    }

    pub async fn listen<Buff>(
        &self,
        endpoint: EndPoint<'_>,
        buff: Buff,
    ) -> core::result::Result<TransportLink<'_, LinkManager, Buff>, TransportLinkError>
    where
        LinkManager: ZLinkManager,
    {
        todo!()
    }
}
