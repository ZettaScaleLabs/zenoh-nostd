use heapless::Vec;
use zenoh_buffer::ZBuf;
use zenoh_protocol::{
    common::{extension::iext, imsg},
    core::encoding::Encoding,
    zenoh::{
        err::{ext, flag, Err},
        id,
    },
};
use zenoh_result::{zbail, zctx, zerr, WithContext, ZE};

use crate::{common::extension, RCodec, WCodec, Zenoh080};

impl<'a, const MAX_EXT_UNKNOWN: usize> WCodec<'a, &Err<'_, MAX_EXT_UNKNOWN>> for Zenoh080 {
    fn write(
        &self,
        message: &Err<'_, MAX_EXT_UNKNOWN>,
        writer: &mut zenoh_buffer::ZBufWriter<'a>,
    ) -> zenoh_result::ZResult<()> {
        let Err {
            encoding,
            ext_sinfo,
            ext_unknown,
            payload,
        } = message;

        let mut header = id::ERR;
        if encoding != &Encoding::empty() {
            header |= flag::E;
        }
        let mut n_exts = (ext_sinfo.is_some() as u8) + (ext_unknown.len() as u8);
        if n_exts != 0 {
            header |= flag::Z;
        }
        self.write(header, writer).ctx(zctx!())?;

        if encoding != &Encoding::empty() {
            self.write(encoding, writer).ctx(zctx!())?;
        }

        if let Some(sinfo) = ext_sinfo.as_ref() {
            n_exts -= 1;
            self.write((sinfo, n_exts != 0), writer).ctx(zctx!())?;
        }
        for u in ext_unknown.iter() {
            n_exts -= 1;
            self.write((u, n_exts != 0), writer).ctx(zctx!())?;
        }

        self.write(payload, writer).ctx(zctx!())?;

        Ok(())
    }
}

impl<'a, const MAX_EXT_UNKNOWN: usize> WCodec<'a, Err<'_, MAX_EXT_UNKNOWN>> for Zenoh080 {
    fn write(
        &self,
        message: Err<'_, MAX_EXT_UNKNOWN>,
        writer: &mut zenoh_buffer::ZBufWriter<'a>,
    ) -> zenoh_result::ZResult<()> {
        self.write(&message, writer).ctx(zctx!())
    }
}

impl<'a, const MAX_EXT_UNKNOWN: usize> RCodec<'a, Err<'a, MAX_EXT_UNKNOWN>> for Zenoh080 {
    fn read_knowing_header(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
        header: u8,
    ) -> zenoh_result::ZResult<Err<'a, MAX_EXT_UNKNOWN>> {
        if imsg::mid(header) != id::ERR {
            zbail!(ZE::ReadFailure);
        }

        let mut encoding = Encoding::empty();
        if imsg::has_flag(header, flag::E) {
            encoding = self.read(reader).ctx(zctx!())?;
        }

        let mut ext_sinfo: Option<ext::SourceInfoType> = None;
        let mut ext_unknown = Vec::<_, MAX_EXT_UNKNOWN>::new();

        let mut has_ext = imsg::has_flag(header, flag::Z);
        while has_ext {
            let ext: u8 = self.read(reader).ctx(zctx!())?;
            match iext::eid(ext) {
                ext::SourceInfo::ID => {
                    let (s, ext): (ext::SourceInfoType, bool) =
                        self.read_knowing_header(reader, ext)?;
                    ext_sinfo = Some(s);
                    has_ext = ext;
                }
                _ => {
                    let (u, ext) = extension::read(reader, "Err", ext)?;
                    ext_unknown.push(u).map_err(|_| zerr!(ZE::ReadFailure))?;
                    has_ext = ext;
                }
            }
        }

        let payload: ZBuf = self.read(reader).ctx(zctx!())?;

        Ok(Err {
            encoding,
            ext_sinfo,
            ext_unknown,
            payload,
        })
    }

    fn read(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
    ) -> zenoh_result::ZResult<Err<'a, MAX_EXT_UNKNOWN>> {
        let header: u8 = self.read(reader).ctx(zctx!())?;
        self.read_knowing_header(reader, header).ctx(zctx!())
    }
}
