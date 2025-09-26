use zenoh_buffers::{
    reader::HasReader,
    writer::{HasWriter, Writer},
    zslice::ZSliceLen,
};
use zenoh_codec::{RCodec, WCodec, Zenoh080};
use zenoh_link::unicast::LinkUnicast;
use zenoh_platform::Platform;
use zenoh_protocol::transport::{BatchSize, TransportMessage};
use zenoh_result::{zerr, ZResult, ZE};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TransportLinkUnicastDirection {
    Inbound,
    Outbound,
}

#[derive(Clone)]
pub struct TransportLinkUnicastConfig {
    pub(crate) mtu: u16,
    pub(crate) is_streamed: bool,
}

pub struct TransportLinkUnicast<T: Platform, const S: usize, const D: usize> {
    pub link: LinkUnicast<T, S, D>,
    pub config: TransportLinkUnicastConfig,

    pub(crate) codec: Zenoh080,
}

impl<T: Platform, const S: usize, const D: usize> TransportLinkUnicast<T, S, D> {
    pub fn new(link: LinkUnicast<T, S, D>, config: TransportLinkUnicastConfig) -> Self {
        Self {
            link,
            config,
            codec: Zenoh080::default(),
        }
    }

    pub fn reconfigure(self, new_config: TransportLinkUnicastConfig) -> Self {
        Self {
            link: self.link,
            config: new_config,
            codec: self.codec,
        }
    }

    pub async fn send<const N: usize>(&mut self, msg: &TransportMessage) -> ZResult<()> {
        let mut slice = zenoh_buffers::vec::empty::<N>();
        let mut writer = slice.writer();

        if self.config.is_streamed {
            let len = BatchSize::MIN.to_le_bytes();
            writer.write(&len).map_err(|_| zerr!(ZE::DidntWrite))?;
        }

        self.codec.write(&mut writer, msg)?;

        if self.config.is_streamed {
            let space = BatchSize::MIN.to_le_bytes().len();
            let payload_len = (slice.len() - space) as BatchSize;

            let len_bytes = payload_len.to_le_bytes();
            slice.as_mut_slice()[..space].copy_from_slice(&len_bytes);
        }

        self.link.write_all(slice.as_slice()).await
    }

    pub async fn recv<const N: usize>(&mut self) -> ZResult<TransportMessage> {
        let mut slice = zenoh_buffers::vec::uninit::<N>();

        if self.config.is_streamed {
            let mut len = BatchSize::MIN.to_le_bytes();
            self.link.read_exact(&mut len).await?;
            let l = BatchSize::from_le_bytes(len) as usize;

            self.link.read_exact(&mut slice.as_mut_slice()[..l]).await?;
        } else {
            self.link.read_exact(slice.as_mut()).await?;
        }

        let mut reader = slice.reader();

        let msg: TransportMessage = (self.codec, ZSliceLen::<N>).read(&mut reader)?;

        Ok(msg)
    }
}
