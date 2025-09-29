use heapless::Vec;
use zenoh_result::{zbail, zerr, ZResult, ZE};

pub trait Writer<'a> {
    fn write(&mut self, buffer: &[u8], len: usize) -> ZResult<usize>;
    fn write_slice(&mut self, buffer: &'a [u8]) -> ZResult<()>;
}

#[derive(Clone, Debug)]
pub enum BufferSlice<'a> {
    Empty,

    InternalSlice { start: usize, len: usize },

    ExternalSlice(&'a [u8]),
}

#[derive(Clone, Debug)]
pub struct Buffer<'a> {
    scratch: Vec<u8, 256>,
    last: usize,

    slices: Vec<BufferSlice<'a>, 256>,
}

impl<'a> Buffer<'a> {
    pub fn new() -> Self {
        Self {
            scratch: Vec::new(),
            last: 0,
            slices: Vec::new(),
        }
    }

    pub fn flush(&mut self) -> ZResult<()> {
        if self.scratch.len() > self.last {
            let slice = BufferSlice::InternalSlice {
                start: self.last,
                len: self.scratch.len() - self.last,
            };

            self.slices
                .push(slice)
                .map_err(|_| zerr!(ZE::CapacityExceeded))?;

            self.last = self.scratch.len();
        }

        Ok(())
    }

    pub fn as_bytes(&self) -> ZResult<Vec<&[u8], 256>> {
        let mut result: Vec<&[u8], 256> = Vec::new();

        for slice in &self.slices {
            match slice {
                BufferSlice::Empty => {}
                BufferSlice::InternalSlice { start, len } => {
                    let start = *start;
                    let len = *len;
                    let end = start + len;
                    if end <= self.scratch.len() {
                        result
                            .push(&self.scratch[start..end])
                            .map_err(|_| zerr!(ZE::CapacityExceeded))?;
                    } else {
                        zbail!(ZE::InvalidArgument);
                    }
                }
                BufferSlice::ExternalSlice(ext) => {
                    result.push(ext).map_err(|_| zerr!(ZE::CapacityExceeded))?;
                }
            }
        }

        Ok(result)
    }
}

impl<'a> Writer<'a> for Buffer<'a> {
    fn write(&mut self, buffer: &[u8], len: usize) -> ZResult<usize> {
        if buffer.len() < len {
            zbail!(ZE::InvalidArgument);
        }

        let current = self.scratch.len();

        let to_write = len.min(256 - current);

        self.scratch
            .extend_from_slice(&buffer[..to_write])
            .map_err(|_| zerr!(ZE::CapacityExceeded))?;

        Ok(to_write)
    }

    fn write_slice(&mut self, buffer: &'a [u8]) -> ZResult<()> {
        self.flush()?;

        self.slices
            .push(BufferSlice::ExternalSlice(buffer))
            .map_err(|_| zerr!(ZE::CapacityExceeded))
    }
}

fn main() -> ZResult<()> {
    let header: [u8; 9] = [
        0x7E, // Start byte
        0x01, // Version
        0x00, 0x00, // Length (to be filled later)
        0x00, 0x01, // Message type (e.g., DATA)
        0x00, 0x00, 0x00, // Reserved
    ];

    let payload = b"Hello, World!";

    let footer = [0x7F, 0x7F]; // End bytes

    let mut buffer = Buffer::new();
    buffer.write(&header, header.len())?;
    buffer.write_slice(payload)?;
    buffer.write(&footer, footer.len())?;

    println!("Buffer: {:?}", buffer);

    buffer.flush()?;
    let bytes = buffer.as_bytes()?;
    println!("As bytes: {:?}", bytes);

    Ok(())
}
