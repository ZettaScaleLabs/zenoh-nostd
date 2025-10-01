use heapless::Vec;
use zenoh_protocol::{
    common::{extension::iext, imsg},
    zenoh::{
        id,
        query::{ext, flag, ConsolidationMode, Query},
    },
};
use zenoh_result::{zbail, zerr, ZE};

use crate::{common::extension, RCodec, WCodec, Zenoh080};

impl<'a> WCodec<'a, &ConsolidationMode> for Zenoh080 {
    fn write(
        &self,
        message: &ConsolidationMode,
        writer: &mut zenoh_buffer::ZBufWriter<'a>,
    ) -> zenoh_result::ZResult<()> {
        let v: u64 = match message {
            ConsolidationMode::Auto => 0,
            ConsolidationMode::None => 1,
            ConsolidationMode::Monotonic => 2,
            ConsolidationMode::Latest => 3,
        };

        self.write(v, writer)
    }
}

impl<'a> RCodec<'a, ConsolidationMode> for Zenoh080 {
    fn read(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
    ) -> zenoh_result::ZResult<ConsolidationMode> {
        let v: u64 = self.read(reader)?;

        match v {
            0 => Ok(ConsolidationMode::Auto),
            1 => Ok(ConsolidationMode::None),
            2 => Ok(ConsolidationMode::Monotonic),
            3 => Ok(ConsolidationMode::Latest),
            _ => Ok(ConsolidationMode::Auto),
        }
    }
}

impl<'a, const MAX_EXT_UNKNOWN: usize> WCodec<'a, &Query<'_, MAX_EXT_UNKNOWN>> for Zenoh080 {
    fn write(
        &self,
        message: &Query<'_, MAX_EXT_UNKNOWN>,
        writer: &mut zenoh_buffer::ZBufWriter<'a>,
    ) -> zenoh_result::ZResult<()> {
        let Query {
            consolidation,
            parameters,
            ext_sinfo,
            ext_body,
            ext_attachment,
            ext_unknown,
        } = message;

        let mut header = id::QUERY;

        if consolidation != &ConsolidationMode::DEFAULT {
            header |= flag::C;
        }

        if !parameters.is_empty() {
            header |= flag::P;
        }

        let mut n_exts = (ext_sinfo.is_some() as u8)
            + (ext_body.is_some() as u8)
            + (ext_attachment.is_some() as u8)
            + (ext_unknown.len() as u8);

        if n_exts != 0 {
            header |= flag::Z;
        }

        self.write(header, writer)?;

        if consolidation != &ConsolidationMode::DEFAULT {
            self.write(consolidation, writer)?;
        }
        if !parameters.is_empty() {
            self.write(*parameters, writer)?;
        }

        if let Some(sinfo) = ext_sinfo.as_ref() {
            n_exts -= 1;
            self.write((sinfo, n_exts != 0), writer)?;
        }
        if let Some(body) = ext_body.as_ref() {
            n_exts -= 1;
            self.write((body, n_exts != 0), writer)?;
        }
        if let Some(att) = ext_attachment.as_ref() {
            n_exts -= 1;
            self.write((att, n_exts != 0), writer)?;
        }
        for u in ext_unknown.iter() {
            n_exts -= 1;
            self.write((u, n_exts != 0), writer)?;
        }

        Ok(())
    }
}

impl<'a, const MAX_EXT_UNKNOWN: usize> RCodec<'a, Query<'a, MAX_EXT_UNKNOWN>> for Zenoh080 {
    fn read_knowing_header(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
        header: u8,
    ) -> zenoh_result::ZResult<Query<'a, MAX_EXT_UNKNOWN>> {
        if imsg::mid(header) != id::QUERY {
            zbail!(ZE::ReadFailure);
        }

        let mut consolidation = ConsolidationMode::DEFAULT;
        if imsg::has_flag(header, flag::C) {
            consolidation = self.read(reader)?;
        }

        let mut parameters = "";
        if imsg::has_flag(header, flag::P) {
            parameters = self.read(reader)?;
        }

        let mut ext_sinfo: Option<ext::SourceInfoType> = None;
        let mut ext_body: Option<ext::QueryBodyType> = None;
        let mut ext_attachment: Option<ext::AttachmentType> = None;
        let mut ext_unknown = Vec::new();

        let mut has_ext = imsg::has_flag(header, flag::Z);
        while has_ext {
            let ext: u8 = self.read(reader)?;

            match iext::eid(ext) {
                ext::SourceInfo::ID => {
                    let (s, ext): (ext::SourceInfoType, bool) =
                        self.read_knowing_header(reader, ext)?;

                    ext_sinfo = Some(s);
                    has_ext = ext;
                }
                ext::QueryBodyType::SID | ext::QueryBodyType::VID => {
                    let (s, ext): (ext::QueryBodyType, bool) =
                        self.read_knowing_header(reader, ext)?;

                    ext_body = Some(s);
                    has_ext = ext;
                }
                ext::Attachment::ID => {
                    let (a, ext): (ext::AttachmentType, bool) =
                        self.read_knowing_header(reader, ext)?;

                    ext_attachment = Some(a);
                    has_ext = ext;
                }
                _ => {
                    let (u, ext) = extension::read(reader, "Query", ext)?;
                    ext_unknown
                        .push(u)
                        .map_err(|_| zerr!(ZE::CapacityExceeded))?;
                    has_ext = ext;
                }
            }
        }

        Ok(Query {
            consolidation,
            parameters,
            ext_sinfo,
            ext_body,
            ext_attachment,
            ext_unknown,
        })
    }

    fn read(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
    ) -> zenoh_result::ZResult<Query<'a, MAX_EXT_UNKNOWN>> {
        let header: u8 = self.read(reader)?;

        self.read_knowing_header(reader, header)
    }
}
