use zenoh_buffers::{BBuf, ZSlice, ZSliceBuffer};
use zenoh_link::unicast::LinkUnicast;
use zenoh_protocol::transport::{BatchSize, TransportMessage};
use zenoh_result::{zerror, ZResult};

use crate::common::batch::{BatchConfig, Decode, Encode, Finalize, RBatch, WBatch};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TransportLinkUnicastDirection {
    Inbound,
    Outbound,
}

#[derive(Clone, Debug)]
pub struct TransportLinkUnicastConfig {
    pub(crate) mtu: u16,
    pub(crate) is_streamed: bool,
}

pub struct TransportLinkUnicast {
    pub link: LinkUnicast,
    pub config: TransportLinkUnicastConfig,
    pub buffer: Option<BBuf>,
}

impl TransportLinkUnicast {
    pub fn new(link: LinkUnicast, config: TransportLinkUnicastConfig) -> Self {
        Self {
            link,
            config,
            buffer: None,
        }
    }

    pub fn reconfigure(self, new_config: TransportLinkUnicastConfig) -> Self {
        Self {
            link: self.link,
            config: new_config,
            buffer: self.buffer,
        }
    }

    pub async fn send_batch(&mut self, batch: &mut WBatch) -> ZResult<()> {
        const ERR: &str = "Write error on link: ";

        let res = batch
            .finalize(self.buffer.as_mut())
            .map_err(|_| zerror!("{ERR}"))?;

        let bytes = match res {
            Finalize::Batch => batch.as_slice(),
            Finalize::Buffer => self
                .buffer
                .as_ref()
                .ok_or_else(|| zerror!("Invalid buffer finalization"))?
                .as_slice(),
        };

        self.link.write_all(bytes).await?;

        Ok(())
    }

    pub async fn send(&mut self, msg: &TransportMessage) -> ZResult<usize> {
        const ERR: &str = "Write error on link: ";

        // Create the batch for serializing the message
        let mut batch = WBatch::new(BatchConfig {
            mtu: self.config.mtu,
            is_streamed: self.config.is_streamed,
        });
        batch.encode(msg).map_err(|_| zerror!("{ERR}"))?;
        let len = batch.len() as usize;
        self.send_batch(&mut batch).await?;
        Ok(len)
    }

    pub async fn recv_batch<C, T>(&mut self, buff: C) -> ZResult<RBatch>
    where
        C: Fn() -> T + Copy,
        T: AsMut<[u8]> + ZSliceBuffer + 'static,
    {
        const ERR: &str = "Read error from link: ";

        let mut into = (buff)();
        let end = if self.link.is_streamed() {
            // Read and decode the message length
            let mut len = BatchSize::MIN.to_le_bytes();
            self.link.read_exact(&mut len).await?;
            let l = BatchSize::from_le_bytes(len) as usize;

            // Read the bytes
            let slice = into
                .as_mut()
                .get_mut(len.len()..len.len() + l)
                .ok_or_else(|| zerror!("{ERR}. Invalid batch length or buffer size."))?;
            self.link.read_exact(slice).await?;
            len.len() + l
        } else {
            // Read the bytes
            self.link.read(into.as_mut()).await?
        };

        let buffer = {
            use std::sync::Arc;
            ZSlice::new(Arc::new(into), 0, end)
                .map_err(|_| zerror!("{ERR}. ZSlice index(es) out of bounds"))?
        };

        let mut batch = RBatch::new(
            BatchConfig {
                mtu: self.config.mtu,
                is_streamed: self.config.is_streamed,
            },
            buffer,
        );
        batch.initialize(buff).map_err(|e| zerror!("{ERR}. {e}."))?;

        Ok(batch)
    }

    pub async fn recv(&mut self) -> ZResult<TransportMessage> {
        let mtu = self.config.mtu as usize;
        let mut batch = self
            .recv_batch(|| zenoh_buffers::vec::uninit(mtu).into_boxed_slice())
            .await?;
        let msg = batch
            .decode()
            .map_err(|_| zerror!("Decode error on link"))?;
        Ok(msg)
    }
}
