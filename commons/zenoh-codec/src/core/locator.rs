use heapless::Vec;
use zenoh_protocol::core::locator::Locator;
use zenoh_result::{zbail, zerr, ZE};

use crate::{RCodec, WCodec, Zenoh080};

impl<'a, const N: usize> WCodec<'a, &Locator<N>> for Zenoh080 {
    fn write(
        &self,
        message: &Locator<N>,
        writer: &mut zenoh_buffer::ZBufWriter<'a>,
    ) -> zenoh_result::ZResult<()> {
        let str = message.as_str();
        if str.len() > 255 {
            zbail!(ZE::InvalidArgument);
        }

        self.write(str, writer)
    }
}

impl<'a, const N: usize> WCodec<'a, Locator<N>> for crate::Zenoh080 {
    fn write(
        &self,
        message: Locator<N>,
        writer: &mut zenoh_buffer::ZBufWriter<'a>,
    ) -> zenoh_result::ZResult<()> {
        self.write(&message, writer)
    }
}

impl<'a, const N: usize> RCodec<'a, Locator<N>> for crate::Zenoh080 {
    fn read(&self, reader: &mut zenoh_buffer::ZBufReader<'a>) -> zenoh_result::ZResult<Locator<N>> {
        let str: &str = self.read(reader)?;
        Locator::try_from(str).map_err(|_| zerr!(ZE::ReadFailure))
    }
}

impl<'a, const N: usize> WCodec<'a, &[Locator<N>]> for crate::Zenoh080 {
    fn write(
        &self,
        message: &[Locator<N>],
        writer: &mut zenoh_buffer::ZBufWriter<'a>,
    ) -> zenoh_result::ZResult<()> {
        self.write(message.len(), writer)?;

        for locator in message {
            self.write(locator, writer)?;
        }

        Ok(())
    }
}

impl<'a, const L: usize, const N: usize> RCodec<'a, Vec<Locator<N>, L>> for crate::Zenoh080 {
    fn read(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
    ) -> zenoh_result::ZResult<Vec<Locator<N>, L>> {
        let len: usize = self.read(reader)?;
        if len > L {
            zbail!(ZE::ReadFailure);
        }

        let mut vec: Vec<Locator<N>, L> = Vec::new();
        for _ in 0..len {
            let locator: Locator<N> = self.read(reader)?;
            vec.push(locator).map_err(|_| zerr!(ZE::ReadFailure))?;
        }

        Ok(vec)
    }
}
