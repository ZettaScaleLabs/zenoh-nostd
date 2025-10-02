use heapless::Vec;
use zenoh_protocol::core::locator::Locator;
use zenoh_result::{zbail, zctx, zerr, WithContext, ZE};

use crate::{RCodec, WCodec, ZCodec};

impl<'a, const N: usize> WCodec<'a, &Locator<N>> for ZCodec {
    fn write(
        &self,
        message: &Locator<N>,
        writer: &mut zenoh_buffer::ZBufWriter<'a>,
    ) -> zenoh_result::ZResult<()> {
        let str = message.as_str();
        if str.len() > 255 {
            zbail!(ZE::InvalidArgument);
        }

        self.write(str, writer).ctx(zctx!())
    }
}

impl<'a, const N: usize> WCodec<'a, Locator<N>> for crate::ZCodec {
    fn write(
        &self,
        message: Locator<N>,
        writer: &mut zenoh_buffer::ZBufWriter<'a>,
    ) -> zenoh_result::ZResult<()> {
        self.write(&message, writer).ctx(zctx!())
    }
}

impl<'a, const N: usize> RCodec<'a, Locator<N>> for crate::ZCodec {
    fn read(&self, reader: &mut zenoh_buffer::ZBufReader<'a>) -> zenoh_result::ZResult<Locator<N>> {
        let str: &str = self.read(reader).ctx(zctx!())?;
        Locator::try_from(str).map_err(|_| zerr!(ZE::ReadFailure))
    }
}

impl<'a, const N: usize> WCodec<'a, &[Locator<N>]> for crate::ZCodec {
    fn write(
        &self,
        message: &[Locator<N>],
        writer: &mut zenoh_buffer::ZBufWriter<'a>,
    ) -> zenoh_result::ZResult<()> {
        self.write(message.len(), writer).ctx(zctx!())?;

        for locator in message {
            self.write(locator, writer).ctx(zctx!())?;
        }

        Ok(())
    }
}

impl<'a, const L: usize, const N: usize> RCodec<'a, Vec<Locator<N>, L>> for crate::ZCodec {
    fn read(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
    ) -> zenoh_result::ZResult<Vec<Locator<N>, L>> {
        let len: usize = self.read(reader).ctx(zctx!())?;
        if len > L {
            zbail!(ZE::ReadFailure);
        }

        let mut vec: Vec<Locator<N>, L> = Vec::new();
        for _ in 0..len {
            let locator: Locator<N> = self.read(reader).ctx(zctx!())?;
            vec.push(locator).map_err(|_| zerr!(ZE::ReadFailure))?;
        }

        Ok(vec)
    }
}
