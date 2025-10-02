use zenoh_protocol::{
    common::{extension::iext, imsg},
    zenoh::{
        id,
        query::{ext, flag, ConsolidationMode, Query},
    },
};
use zenoh_result::{zbail, zctx, WithContext, ZE};

use crate::{common::extension, RCodec, WCodec, ZCodec};

impl<'a> WCodec<'a, &ConsolidationMode> for ZCodec {
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

        self.write(v, writer).ctx(zctx!())
    }
}

impl<'a> RCodec<'a, ConsolidationMode> for ZCodec {
    fn read(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
    ) -> zenoh_result::ZResult<ConsolidationMode> {
        let v: u64 = self.read(reader).ctx(zctx!())?;

        match v {
            0 => Ok(ConsolidationMode::Auto),
            1 => Ok(ConsolidationMode::None),
            2 => Ok(ConsolidationMode::Monotonic),
            3 => Ok(ConsolidationMode::Latest),
            _ => Ok(ConsolidationMode::Auto),
        }
    }
}

impl<'a> WCodec<'a, &Query<'_>> for ZCodec {
    fn write(
        &self,
        message: &Query<'_>,
        writer: &mut zenoh_buffer::ZBufWriter<'a>,
    ) -> zenoh_result::ZResult<()> {
        let Query {
            consolidation,
            parameters,
            ext_sinfo,
            ext_body,
            ext_attachment,
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
            + (ext_attachment.is_some() as u8);

        if n_exts != 0 {
            header |= flag::Z;
        }

        self.write(header, writer).ctx(zctx!())?;

        if consolidation != &ConsolidationMode::DEFAULT {
            self.write(consolidation, writer).ctx(zctx!())?;
        }
        if !parameters.is_empty() {
            self.write(*parameters, writer).ctx(zctx!())?;
        }

        if let Some(sinfo) = ext_sinfo.as_ref() {
            n_exts -= 1;
            self.write((sinfo, n_exts != 0), writer).ctx(zctx!())?;
        }
        if let Some(body) = ext_body.as_ref() {
            n_exts -= 1;
            self.write((body, n_exts != 0), writer).ctx(zctx!())?;
        }
        if let Some(att) = ext_attachment.as_ref() {
            n_exts -= 1;
            self.write((att, n_exts != 0), writer).ctx(zctx!())?;
        }

        Ok(())
    }
}

impl<'a> RCodec<'a, Query<'a>> for ZCodec {
    fn read_knowing_header(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
        header: u8,
    ) -> zenoh_result::ZResult<Query<'a>> {
        if imsg::mid(header) != id::QUERY {
            zbail!(ZE::ReadFailure);
        }

        let mut consolidation = ConsolidationMode::DEFAULT;
        if imsg::has_flag(header, flag::C) {
            consolidation = self.read(reader).ctx(zctx!())?;
        }

        let mut parameters = "";
        if imsg::has_flag(header, flag::P) {
            parameters = self.read(reader).ctx(zctx!())?;
        }

        let mut ext_sinfo: Option<ext::SourceInfoType> = None;
        let mut ext_body: Option<ext::QueryBodyType> = None;
        let mut ext_attachment: Option<ext::AttachmentType> = None;

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
                    let (_, ext) = extension::read(reader, "Query", ext)?;
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
        })
    }

    fn read(&self, reader: &mut zenoh_buffer::ZBufReader<'a>) -> zenoh_result::ZResult<Query<'a>> {
        let header: u8 = self.read(reader).ctx(zctx!())?;

        self.read_knowing_header(reader, header).ctx(zctx!())
    }
}
