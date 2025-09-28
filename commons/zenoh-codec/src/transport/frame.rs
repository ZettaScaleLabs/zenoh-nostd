use zenoh_buffers::{
    reader::{BacktrackableReader, Reader},
    writer::Writer,
    zslice::ZSliceLen,
};
use zenoh_protocol::{
    common::{iext, imsg},
    core::Reliability,
    transport::{
        frame::{ext, flag, Frame, FrameHeader},
        id, TransportSn,
    },
};
use zenoh_result::{bail, ZError, ZResult, ZE};

use crate::{common::extension, RCodec, WCodec, Zenoh080, Zenoh080Header};

// FrameHeader
impl<W> WCodec<&FrameHeader, &mut W> for Zenoh080
where
    W: Writer,
{
    type Output = ZResult<()>;

    fn write(self, writer: &mut W, x: &FrameHeader) -> Self::Output {
        let FrameHeader {
            reliability,
            sn,
            ext_qos,
        } = x;

        // Header
        let mut header = id::FRAME;
        if let Reliability::Reliable = reliability {
            header |= flag::R;
        }
        if ext_qos != &ext::QoSType::DEFAULT {
            header |= flag::Z;
        }
        self.write(&mut *writer, header)?;

        // Body
        self.write(&mut *writer, sn)?;

        // Extensions
        if ext_qos != &ext::QoSType::DEFAULT {
            self.write(&mut *writer, (x.ext_qos, false))?;
        }

        Ok(())
    }
}

impl<R> RCodec<FrameHeader, &mut R> for Zenoh080
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<FrameHeader> {
        let header: u8 = self.read(&mut *reader)?;
        let codec = Zenoh080Header::new(header);
        codec.read(reader)
    }
}

impl<R> RCodec<FrameHeader, &mut R> for Zenoh080Header
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<FrameHeader> {
        if imsg::mid(self.header) != id::FRAME {
            bail!(ZE::DidntRead)
        }

        let reliability = match imsg::has_flag(self.header, flag::R) {
            true => Reliability::Reliable,
            false => Reliability::BestEffort,
        };
        let sn: TransportSn = self.codec.read(&mut *reader)?;

        // Extensions
        let mut ext_qos = ext::QoSType::DEFAULT;

        let mut has_ext = imsg::has_flag(self.header, flag::Z);
        while has_ext {
            let ext: u8 = self.codec.read(&mut *reader)?;
            let eodec = Zenoh080Header::new(ext);
            match iext::eid(ext) {
                ext::QoS::ID => {
                    let (q, ext): (ext::QoSType, bool) = eodec.read(&mut *reader)?;
                    ext_qos = q;
                    has_ext = ext;
                }
                _ => {
                    has_ext = extension::skip::<_, 1, 32>(reader, "Frame", ext)?;
                }
            }
        }

        Ok(FrameHeader {
            reliability,
            sn,
            ext_qos,
        })
    }
}

// Frame
impl<W> WCodec<&Frame, &mut W> for Zenoh080
where
    W: Writer,
{
    type Output = ZResult<()>;

    fn write(self, writer: &mut W, x: &Frame) -> Self::Output {
        let Frame {
            reliability,
            sn,
            payload,
            ext_qos,
        } = x;

        // Header
        let header = FrameHeader {
            reliability: *reliability,
            sn: *sn,
            ext_qos: *ext_qos,
        };
        self.write(&mut *writer, &header)?;

        // Body
        writer.write_zslice(payload)?;

        Ok(())
    }
}

impl<R, const L: usize> RCodec<Frame, &mut R> for (Zenoh080, ZSliceLen<L>)
where
    R: Reader + BacktrackableReader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<Frame> {
        let header: u8 = self.0.read(&mut *reader)?;
        let codec = (Zenoh080Header::new(header), ZSliceLen::<L>);
        codec.read(reader)
    }
}

impl<R, const L: usize> RCodec<Frame, &mut R> for (Zenoh080Header, ZSliceLen<L>)
where
    R: Reader + BacktrackableReader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<Frame> {
        let header: FrameHeader = self.0.read(&mut *reader)?;
        let payload = reader.read_zslice::<L>(reader.remaining())?;

        Ok(Frame {
            reliability: header.reliability,
            sn: header.sn,
            ext_qos: header.ext_qos,
            payload,
        })
    }
}
