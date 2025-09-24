use core::num::NonZeroUsize;

use zenoh_buffers::{
    buffer::Buffer,
    reader::{DidntRead, HasReader},
    writer::{DidntWrite, HasWriter, Writer},
    BBuf, ZBufReader, ZSlice, ZSliceBuffer,
};

use zenoh_codec::{
    transport::batch::{BatchError, Zenoh080Batch},
    RCodec, WCodec,
};

use zenoh_protocol::{
    network::NetworkMessageRef,
    transport::{fragment::FragmentHeader, frame::FrameHeader, BatchSize, TransportMessage},
};

use zenoh_result::{zerror, ZResult};

const L_LEN: usize = (BatchSize::BITS / 8) as usize;
const H_LEN: usize = BatchHeader::SIZE;

// Split the inner buffer into (length, header, payload) immutable slices
macro_rules! zsplit {
    ($slice:expr, $config:expr) => {{
        match ($config.is_streamed, $config.has_header()) {
            (true, true) => {
                let (l, s) = $slice.split_at(L_LEN);
                let (h, p) = s.split_at(H_LEN);
                (l, h, p)
            }
            (true, false) => {
                let (l, p) = $slice.split_at(L_LEN);
                (l, &[], p)
            }
            (false, true) => {
                let (h, p) = $slice.split_at(H_LEN);
                (&[], h, p)
            }
            (false, false) => (&[], &[], $slice),
        }
    }};
}

macro_rules! zsplit_mut {
    ($slice:expr, $config:expr) => {{
        match ($config.is_streamed, $config.has_header()) {
            (true, true) => {
                let (l, s) = $slice.split_at_mut(L_LEN);
                let (h, p) = s.split_at_mut(H_LEN);
                (l, h, p)
            }
            (true, false) => {
                let (l, p) = $slice.split_at_mut(L_LEN);
                (l, &mut [], p)
            }
            (false, true) => {
                let (h, p) = $slice.split_at_mut(H_LEN);
                (&mut [], h, p)
            }
            (false, false) => (&mut [], &mut [], $slice),
        }
    }};
}

// Batch config
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct BatchConfig {
    pub mtu: BatchSize,
    pub is_streamed: bool,
}

impl Default for BatchConfig {
    fn default() -> Self {
        BatchConfig {
            mtu: BatchSize::MAX,
            is_streamed: false,
        }
    }
}

impl BatchConfig {
    const fn has_header(&self) -> bool {
        false
    }

    fn header(&self) -> Option<BatchHeader> {
        None
    }
}

// Batch header
#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct BatchHeader(u8);

impl BatchHeader {
    const SIZE: usize = 1;
    const fn as_u8(&self) -> u8 {
        self.0
    }
}

#[repr(u8)]
#[derive(Debug)]
pub enum Finalize {
    Batch,
    Buffer,
}

/// Write Batch
///
/// A [`WBatch`] is a non-expandable and contiguous region of memory
/// that is used to serialize [`TransportMessage`] and [`NetworkMessage`].
///
/// [`TransportMessage`] are always serialized on the batch as they are, while
/// [`NetworkMessage`] are always serializaed on the batch as part of a [`TransportMessage`]
/// [TransportMessage] Frame. Reliable and Best Effort Frames can be interleaved on the same
/// [`WBatch`] as long as they fit in the remaining buffer capacity.
///
/// In the serialized form, the [`WBatch`] always contains one or more
/// [`TransportMessage`]. In the particular case of [`TransportMessage`] Frame,
/// its payload is either (i) one or more complete [`NetworkMessage`] or (ii) a fragment of a
/// a [`NetworkMessage`].
///
/// As an example, the content of the [`WBatch`] in memory could be:
///
/// | Keep Alive | Frame Reliable\<Zenoh Message, Zenoh Message\> | Frame Best Effort\<Zenoh Message Fragment\> |
///
/// [`NetworkMessage`]: zenoh_protocol::network::NetworkMessage
#[derive(Clone, Debug)]
pub struct WBatch {
    // The buffer to perform the batching on
    pub buffer: BBuf,
    // The batch codec
    pub codec: Zenoh080Batch,
    // It contains 1 byte as additional header, e.g. to signal the batch is compressed
    pub config: BatchConfig,
    // an ephemeral batch will not be recycled in the pipeline
    // it can be used to push a stop fragment when no batch are available
    pub ephemeral: bool,
}

impl WBatch {
    pub fn new(config: BatchConfig) -> Self {
        let mut batch = Self {
            buffer: BBuf::with_capacity(config.mtu as usize),
            codec: Zenoh080Batch::new(),
            config,
            ephemeral: false,
        };

        // Bring the batch in a clear state
        batch.clear();

        batch
    }

    pub fn new_ephemeral(config: BatchConfig) -> Self {
        Self {
            ephemeral: true,
            ..Self::new(config)
        }
    }

    pub fn is_ephemeral(&self) -> bool {
        self.ephemeral
    }

    /// Verify that the [`WBatch`] has no serialized bytes.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the total number of bytes that have been serialized on the [`WBatch`].
    #[inline(always)]
    pub fn len(&self) -> BatchSize {
        let (_l, _h, p) = Self::split(self.buffer.as_slice(), &self.config);
        p.len() as BatchSize
    }

    /// Clear the [`WBatch`] memory buffer and related internal state.
    #[inline(always)]
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.codec.clear();
        Self::init(&mut self.buffer, &self.config);
    }

    /// Get a `&[u8]` to access the internal memory buffer, usually for transmitting it on the network.
    #[inline(always)]
    pub fn as_slice(&self) -> &[u8] {
        self.buffer.as_slice()
    }

    fn init(buffer: &mut BBuf, config: &BatchConfig) {
        let mut writer = buffer.writer();
        if config.is_streamed {
            let _ = writer.write_exact(&BatchSize::MIN.to_be_bytes());
        }
        if let Some(h) = config.header() {
            let _ = writer.write_u8(h.as_u8());
        }
    }

    // Split (length, header, payload) internal buffer slice
    #[inline(always)]
    fn split<'a>(buffer: &'a [u8], config: &BatchConfig) -> (&'a [u8], &'a [u8], &'a [u8]) {
        zsplit!(buffer, config)
    }

    // Split (length, header, payload) internal buffer slice
    #[inline(always)]
    fn split_mut<'a>(
        buffer: &'a mut [u8],
        config: &BatchConfig,
    ) -> (&'a mut [u8], &'a mut [u8], &'a mut [u8]) {
        zsplit_mut!(buffer, config)
    }

    pub fn finalize(&mut self, mut buffer: Option<&mut BBuf>) -> ZResult<Finalize> {
        #[allow(unused_mut)]
        let mut res = Finalize::Batch;

        if self.config.is_streamed {
            let buff = match res {
                Finalize::Batch => self.buffer.as_mut_slice(),
                Finalize::Buffer => buffer
                    .as_mut()
                    .ok_or_else(|| zerror!("Support buffer not provided"))?
                    .as_mut_slice(),
            };
            let (length, header, payload) = Self::split_mut(buff, &self.config);
            let len: BatchSize = (header.len() as BatchSize) + (payload.len() as BatchSize);
            length.copy_from_slice(&len.to_le_bytes());
        }

        Ok(res)
    }
}

pub trait Encode<Message> {
    type Output;

    fn encode(self, x: Message) -> Self::Output;
}

impl Encode<&TransportMessage> for &mut WBatch {
    type Output = Result<();

    fn encode(self, x: &TransportMessage) -> Self::Output {
        let mut writer = self.buffer.writer();

        self.codec.write(&mut writer, x)
    }
}

impl Encode<NetworkMessageRef<'_>> for &mut WBatch {
    type Output = Result<(), BatchError>;

    fn encode(self, x: NetworkMessageRef) -> Self::Output {
        let mut writer = self.buffer.writer();
        self.codec.write(&mut writer, x)
    }
}

impl Encode<(NetworkMessageRef<'_>, &FrameHeader)> for &mut WBatch {
    type Output = Result<(), BatchError>;

    fn encode(self, x: (NetworkMessageRef, &FrameHeader)) -> Self::Output {
        let mut writer = self.buffer.writer();

        self.codec.write(&mut writer, x)
    }
}

impl Encode<(&mut ZBufReader<'_>, &mut FragmentHeader)> for &mut WBatch {
    type Output = Result<NonZeroUsize;

    fn encode(self, x: (&mut ZBufReader<'_>, &mut FragmentHeader)) -> Self::Output {
        let mut writer = self.buffer.writer();

        self.codec.write(&mut writer, x)
    }
}

// Read batch
#[derive(Debug)]
pub struct RBatch {
    // The buffer to perform deserializationn from
    buffer: ZSlice,
    // The batch codec
    codec: Zenoh080Batch,
    // The batch config
    config: BatchConfig,
}

impl RBatch {
    pub fn new<T>(config: BatchConfig, buffer: T) -> Self
    where
        T: Into<ZSlice>,
    {
        Self {
            buffer: buffer.into(),
            codec: Zenoh080Batch::new(),
            config,
        }
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    // Split (length, header, payload) internal buffer slice
    #[inline(always)]
    fn split<'a>(buffer: &'a [u8], config: &BatchConfig) -> (&'a [u8], &'a [u8], &'a [u8]) {
        zsplit!(buffer, config)
    }

    pub fn initialize<C, T>(&mut self, #[allow(unused_variables)] buff: C) -> ZResult<()>
    where
        C: Fn() -> T + Copy,
        T: AsMut<[u8]> + ZSliceBuffer + 'static,
    {
        #[allow(unused_variables)]
        let (l, h, p) = Self::split(self.buffer.as_slice(), &self.config);

        self.buffer = self
            .buffer
            .subslice(l.len() + h.len()..self.buffer.len())
            .ok_or_else(|| zerror!("Invalid batch length"))?;

        Ok(())
    }
}

pub trait Decode<Message> {
    type Error;

    fn decode(self) -> Result<Message, Self::Error>;
}

impl Decode<TransportMessage> for &mut RBatch {
    type Error = DidntRead;

    fn decode(self) -> Result<TransportMessage, Self::Error> {
        let mut reader = self.buffer.reader();
        self.codec.read(&mut reader)
    }
}

impl Decode<(TransportMessage, BatchSize)> for &mut RBatch {
    type Error = DidntRead;

    fn decode(self) -> Result<(TransportMessage, BatchSize), Self::Error> {
        let len = self.buffer.len() as BatchSize;
        let mut reader = self.buffer.reader();
        let msg = self.codec.read(&mut reader)?;
        let end = self.buffer.len() as BatchSize;
        Ok((msg, len - end))
    }
}
