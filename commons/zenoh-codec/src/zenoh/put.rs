use heapless::Vec;
use zenoh_protocol::{
    common::{extension::iext, imsg},
    core::encoding::Encoding,
    zenoh::{
        id,
        put::{ext, flag, Put},
    },
};
use zenoh_result::{zbail, zerr, ZE};

use crate::{common::extension, RCodec, WCodec};

impl<'a, const MAX_EXT_UNKNOWN: usize> WCodec<'a, &Put<'_, MAX_EXT_UNKNOWN>> for crate::Zenoh080 {
    fn write(
        &self,
        message: &Put<'_, MAX_EXT_UNKNOWN>,
        writer: &mut zenoh_buffer::ZBufWriter<'a>,
    ) -> zenoh_result::ZResult<()> {
        let Put {
            timestamp,
            encoding,
            ext_sinfo,
            ext_attachment,
            ext_unknown,
            payload,
        } = message;

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

        self.write(header, writer)?;

        if let Some(ts) = timestamp.as_ref() {
            self.write(ts, writer)?;
        }

        if encoding != &Encoding::empty() {
            self.write(encoding, writer)?;
        }

        if let Some(sinfo) = ext_sinfo.as_ref() {
            n_exts -= 1;
            self.write((sinfo, n_exts != 0), writer)?;
        }

        if let Some(att) = ext_attachment.as_ref() {
            n_exts -= 1;
            self.write((att, n_exts != 0), writer)?;
        }

        for u in ext_unknown.iter() {
            n_exts -= 1;
            self.write((u, n_exts != 0), writer)?;
        }

        self.write(payload, writer)?;

        Ok(())
    }
}

impl<'a, const MAX_EXT_UNKNOWN: usize> RCodec<'a, Put<'a, MAX_EXT_UNKNOWN>> for crate::Zenoh080 {
    fn read_knowing_header(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
        header: u8,
    ) -> zenoh_result::ZResult<Put<'a, MAX_EXT_UNKNOWN>> {
        if imsg::mid(header) != id::PUT {
            zbail!(ZE::ReadFailure);
        }

        let mut timestamp: Option<uhlc::Timestamp> = None;
        if imsg::has_flag(header, flag::T) {
            timestamp = Some(self.read(reader)?);
        }

        let mut encoding = Encoding::empty();
        if imsg::has_flag(header, flag::E) {
            encoding = self.read(reader)?;
        }

        // Extensions
        let mut ext_sinfo: Option<ext::SourceInfoType> = None;
        let mut ext_attachment: Option<ext::AttachmentType> = None;
        let mut ext_unknown = Vec::new();

        let mut has_ext = imsg::has_flag(header, flag::Z);
        while has_ext {
            let ext: u8 = self.read(&mut *reader)?;

            match iext::eid(ext) {
                ext::SourceInfo::ID => {
                    let (s, ext): (ext::SourceInfoType, bool) =
                        self.read_knowing_header(reader, ext)?;
                    ext_sinfo = Some(s);
                    has_ext = ext;
                }
                ext::Attachment::ID => {
                    let (a, ext): (ext::AttachmentType, bool) =
                        self.read_knowing_header(reader, ext)?;
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

        let payload: zenoh_buffer::ZBuf<'a> = self.read(reader)?;

        Ok(Put {
            timestamp,
            encoding,
            ext_sinfo,
            ext_attachment,
            ext_unknown,
            payload,
        })
    }

    fn read(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
    ) -> zenoh_result::ZResult<Put<'a, MAX_EXT_UNKNOWN>> {
        let header: u8 = self.read(reader)?;
        self.read_knowing_header(reader, header)
    }
}
