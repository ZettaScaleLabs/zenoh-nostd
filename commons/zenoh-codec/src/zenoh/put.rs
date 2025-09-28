use heapless::Vec;
use zenoh_buffers::{
    reader::Reader,
    writer::Writer,
    zslice::{ZSlice, ZSliceLen},
};
use zenoh_protocol::{
    common::{iext, imsg},
    core::Encoding,
    zenoh::{
        id,
        put::{ext, flag, Put},
    },
};
use zenoh_result::{bail, zerr, ZError, ZResult, ZE};

use crate::Zenoh080Bounded;
use crate::{common::extension, RCodec, WCodec, Zenoh080, Zenoh080Header};

impl<W> WCodec<&Put, &mut W> for Zenoh080
where
    W: Writer,
{
    type Output = ZResult<()>;

    fn write(self, writer: &mut W, x: &Put) -> Self::Output {
        let Put {
            timestamp,
            encoding,
            ext_sinfo,
            ext_attachment,
            ext_unknown,
            payload,
        }: &Put = x;

        // Header
        let mut header = id::PUT;
        if timestamp.is_some() {
            header |= flag::T;
        }
        if encoding != &Encoding::empty() {
            header |= flag::E;
        }
        let mut n_exts = (ext_sinfo.is_some()) as u8
            + (ext_attachment.is_some()) as u8
            + (ext_unknown.len() as u8);

        if n_exts != 0 {
            header |= flag::Z;
        }
        self.write(&mut *writer, header)?;

        // Body
        if let Some(ts) = timestamp.as_ref() {
            self.write(&mut *writer, ts)?;
        }
        if encoding != &Encoding::empty() {
            self.write(&mut *writer, encoding)?;
        }

        // Extensions
        if let Some(sinfo) = ext_sinfo.as_ref() {
            n_exts -= 1;
            self.write(&mut *writer, (sinfo, n_exts != 0))?;
        }

        if let Some(att) = ext_attachment.as_ref() {
            n_exts -= 1;
            self.write(&mut *writer, (att, n_exts != 0))?;
        }
        for u in ext_unknown.iter() {
            n_exts -= 1;
            self.write(&mut *writer, (u, n_exts != 0))?;
        }

        let bodec = Zenoh080Bounded::<u32>::new();
        bodec.write(&mut *writer, payload)?;

        Ok(())
    }
}

impl<R, const L: usize> RCodec<Put, &mut R> for (Zenoh080, ZSliceLen<L>)
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<Put> {
        let header: u8 = self.0.read(&mut *reader)?;
        let codec = (Zenoh080Header::new(header), ZSliceLen::<L>);
        codec.read(reader)
    }
}

impl<R, const L: usize> RCodec<Put, &mut R> for (Zenoh080Header, ZSliceLen<L>)
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<Put> {
        if imsg::mid(self.0.header) != id::PUT {
            bail!(ZE::DidntRead);
        }

        // Body
        let mut timestamp: Option<uhlc::Timestamp> = None;
        if imsg::has_flag(self.0.header, flag::T) {
            timestamp = Some(self.0.codec.read(&mut *reader)?);
        }

        let mut encoding = Encoding::empty();
        if imsg::has_flag(self.0.header, flag::E) {
            encoding = (self.0.codec, ZSliceLen::<32>).read(&mut *reader)?;
        }

        // Extensions
        let mut ext_sinfo: Option<ext::SourceInfoType> = None;
        let mut ext_attachment: Option<ext::AttachmentType> = None;
        let mut ext_unknown = Vec::<_, 8>::new();

        let mut has_ext = imsg::has_flag(self.0.header, flag::Z);
        while has_ext {
            let ext: u8 = self.0.codec.read(&mut *reader)?;
            let eodec = Zenoh080Header::new(ext);
            match iext::eid(ext) {
                ext::SourceInfo::ID => {
                    let (s, ext): (ext::SourceInfoType, bool) = eodec.read(&mut *reader)?;
                    ext_sinfo = Some(s);
                    has_ext = ext;
                }
                ext::Attachment::ID => {
                    let (a, ext): (ext::AttachmentType, bool) = eodec.read(&mut *reader)?;
                    ext_attachment = Some(a);
                    has_ext = ext;
                }
                _ => {
                    let (u, ext) = extension::read(reader, "Put", ext)?;
                    ext_unknown
                        .push(u)
                        .map_err(|_| zerr!(ZE::CapacityExceeded))?;
                    has_ext = ext;
                }
            }
        }

        // Payload
        let payload: ZSlice = {
            let bodec = (Zenoh080Bounded::<u32>::new(), ZSliceLen::<L>);
            bodec.read(&mut *reader)?
        };

        Ok(Put {
            timestamp,
            encoding,
            ext_sinfo,
            ext_attachment,
            ext_unknown,
            payload,
        })
    }
}
