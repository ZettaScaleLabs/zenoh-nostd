use zenoh_buffers::{reader::Reader, writer::Writer, zslice::ZSliceLen};
use zenoh_protocol::{
    common::{iext, imsg},
    core::WireExpr,
    network::{
        id,
        push::Push,
        push::{ext, flag},
        Mapping,
    },
    zenoh::PushBody,
};
use zenoh_result::{bail, ZError, ZResult, ZE};

use crate::{common::extension, RCodec, WCodec, Zenoh080, Zenoh080Condition, Zenoh080Header};

impl<W> WCodec<&Push, &mut W> for Zenoh080
where
    W: Writer,
{
    type Output = ZResult<()>;

    fn write(self, writer: &mut W, x: &Push) -> Self::Output {
        let Push {
            wire_expr,
            ext_qos,
            ext_tstamp,
            ext_nodeid,
            payload,
        } = x;

        // Header
        let mut header = id::PUSH;
        let mut n_exts = ((ext_qos != &ext::QoSType::DEFAULT) as u8)
            + (ext_tstamp.is_some() as u8)
            + ((ext_nodeid != &ext::NodeIdType::DEFAULT) as u8);
        if n_exts != 0 {
            header |= flag::Z;
        }
        if wire_expr.mapping != Mapping::DEFAULT {
            header |= flag::M;
        }
        if wire_expr.has_suffix() {
            header |= flag::N;
        }
        self.write(&mut *writer, header)?;

        // Body
        self.write(&mut *writer, wire_expr)?;

        // Extensions
        if ext_qos != &ext::QoSType::DEFAULT {
            n_exts -= 1;
            self.write(&mut *writer, (*ext_qos, n_exts != 0))?;
        }
        if let Some(ts) = ext_tstamp.as_ref() {
            n_exts -= 1;
            self.write(&mut *writer, (ts, n_exts != 0))?;
        }
        if ext_nodeid != &ext::NodeIdType::DEFAULT {
            n_exts -= 1;
            self.write(&mut *writer, (*ext_nodeid, n_exts != 0))?;
        }

        // Payload
        self.write(&mut *writer, payload)?;

        Ok(())
    }
}

impl<R, const L: usize> RCodec<Push, &mut R> for (Zenoh080, ZSliceLen<L>)
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<Push> {
        let header: u8 = self.0.read(&mut *reader)?;
        let codec = (Zenoh080Header::new(header), ZSliceLen::<L>);
        codec.read(reader)
    }
}

impl<R, const L: usize> RCodec<Push, &mut R> for (Zenoh080Header, ZSliceLen<L>)
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<Push> {
        if imsg::mid(self.0.header) != id::PUSH {
            bail!(ZE::DidntRead);
        }

        // Body
        let ccond = Zenoh080Condition::new(imsg::has_flag(self.0.header, flag::N));
        let mut wire_expr: WireExpr<'static, 32> = ccond.read(&mut *reader)?;
        wire_expr.mapping = if imsg::has_flag(self.0.header, flag::M) {
            Mapping::Sender
        } else {
            Mapping::Receiver
        };

        // Extensions
        let mut ext_qos = ext::QoSType::DEFAULT;
        let mut ext_tstamp = None;
        let mut ext_nodeid = ext::NodeIdType::DEFAULT;

        let mut has_ext = imsg::has_flag(self.0.header, flag::Z);
        while has_ext {
            let ext: u8 = self.0.codec.read(&mut *reader)?;
            let eodec = Zenoh080Header::new(ext);
            match iext::eid(ext) {
                ext::QoS::ID => {
                    let (q, ext): (ext::QoSType, bool) = eodec.read(&mut *reader)?;
                    ext_qos = q;
                    has_ext = ext;
                }
                ext::Timestamp::ID => {
                    let (t, ext): (ext::TimestampType, bool) = eodec.read(&mut *reader)?;
                    ext_tstamp = Some(t);
                    has_ext = ext;
                }
                ext::NodeId::ID => {
                    let (nid, ext): (ext::NodeIdType, bool) = eodec.read(&mut *reader)?;
                    ext_nodeid = nid;
                    has_ext = ext;
                }
                _ => {
                    has_ext = extension::skip::<_, 1, 32>(reader, "Push", ext)?;
                }
            }
        }

        // Payload
        let payload: PushBody = (self.0.codec, ZSliceLen::<L>).read(&mut *reader)?;

        Ok(Push {
            wire_expr,
            payload,
            ext_qos,
            ext_tstamp,
            ext_nodeid,
        })
    }
}
