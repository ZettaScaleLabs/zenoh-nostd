use zenoh_buffer::ZBuf;
use zenoh_protocol::{
    common::{extension::iext, imsg},
    core::{resolution::Resolution, whatami::WhatAmI, ZenohIdProto},
    transport::{
        batch_size, id,
        init::{ext, flag, InitAck, InitSyn},
        BatchSize,
    },
};
use zenoh_result::{zbail, zctx, WithContext, ZE};

use crate::{common::extension, RCodec, WCodec, Zenoh080};

impl<'a> WCodec<'a, &InitSyn<'_>> for Zenoh080 {
    fn write(
        &self,
        message: &InitSyn<'_>,
        writer: &mut zenoh_buffer::ZBufWriter<'a>,
    ) -> zenoh_result::ZResult<()> {
        let InitSyn {
            version,
            whatami,
            zid,
            resolution,
            batch_size,
            ext_qos,
            ext_qos_link,
            ext_auth,
            ext_mlink,
            ext_lowlatency,
            ext_compression,
            ext_patch,
        } = message;

        let mut header = id::INIT;
        if resolution != &Resolution::default() || batch_size != &batch_size::UNICAST {
            header |= flag::S;
        }
        let mut n_exts = (ext_qos.is_some() as u8)
            + (ext_qos_link.is_some() as u8)
            + (ext_auth.is_some() as u8)
            + (ext_mlink.is_some() as u8)
            + (ext_lowlatency.is_some() as u8)
            + (ext_compression.is_some() as u8)
            + (*ext_patch != ext::PatchType::NONE) as u8;

        if n_exts != 0 {
            header |= flag::Z;
        }
        self.write(header, writer).ctx(zctx!())?;
        self.write(*version, writer).ctx(zctx!())?;

        let whatami: u8 = match whatami {
            WhatAmI::Router => 0b00,
            WhatAmI::Peer => 0b01,
            WhatAmI::Client => 0b10,
        };

        let flags: u8 = ((zid.size() as u8 - 1) << 4) | whatami;
        self.write(flags, writer).ctx(zctx!())?;
        self.write_without_length(zid, writer).ctx(zctx!())?;

        if imsg::has_flag(header, flag::S) {
            self.write(resolution.as_u8(), writer).ctx(zctx!())?;
            self.write(batch_size.to_le_bytes(), writer).ctx(zctx!())?;
        }

        if let Some(qos) = ext_qos.as_ref() {
            n_exts -= 1;
            self.write((qos, n_exts != 0), writer).ctx(zctx!())?;
        }

        if let Some(qos_link) = ext_qos_link.as_ref() {
            n_exts -= 1;
            self.write((qos_link, n_exts != 0), writer).ctx(zctx!())?;
        }

        if let Some(auth) = ext_auth.as_ref() {
            n_exts -= 1;
            self.write((auth, n_exts != 0), writer).ctx(zctx!())?;
        }

        if let Some(mlink) = ext_mlink.as_ref() {
            n_exts -= 1;
            self.write((mlink, n_exts != 0), writer).ctx(zctx!())?;
        }

        if let Some(lowlatency) = ext_lowlatency.as_ref() {
            n_exts -= 1;
            self.write((lowlatency, n_exts != 0), writer).ctx(zctx!())?;
        }

        if let Some(compression) = ext_compression.as_ref() {
            n_exts -= 1;
            self.write((compression, n_exts != 0), writer)
                .ctx(zctx!())?;
        }

        if *ext_patch != ext::PatchType::NONE {
            n_exts -= 1;
            self.write((*ext_patch, n_exts != 0), writer).ctx(zctx!())?;
        }

        Ok(())
    }
}

impl<'a> RCodec<'a, InitSyn<'a>> for Zenoh080 {
    fn read_knowing_header(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
        header: u8,
    ) -> zenoh_result::ZResult<InitSyn<'a>> {
        if imsg::mid(header) != id::INIT || imsg::has_flag(header, flag::A) {
            zbail!(ZE::ReadFailure);
        }

        let version: u8 = self.read(reader).ctx(zctx!())?;
        let flags: u8 = self.read(reader).ctx(zctx!())?;
        let whatami = match flags & 0b11 {
            0b00 => WhatAmI::Router,
            0b01 => WhatAmI::Peer,
            0b10 => WhatAmI::Client,
            _ => zbail!(ZE::InvalidBits),
        };

        let length = 1 + ((flags >> 4) as usize);
        let zid: ZenohIdProto = self.read_knowing_length(reader, length).ctx(zctx!())?;

        let mut resolution = Resolution::default();
        let mut batch_size = batch_size::UNICAST.to_le_bytes();

        if imsg::has_flag(header, flag::S) {
            let flags: u8 = self.read(reader).ctx(zctx!())?;
            resolution = Resolution::from(flags & 0b00111111);
            batch_size = self.read(reader).ctx(zctx!())?;
        }

        let batch_size = BatchSize::from_le_bytes(batch_size);

        let mut ext_qos = None;
        let mut ext_qos_link = None;
        let mut ext_auth = None;
        let mut ext_mlink = None;
        let mut ext_lowlatency = None;
        let mut ext_compression = None;
        let mut ext_patch = ext::PatchType::NONE;

        let mut has_ext = imsg::has_flag(header, flag::Z);

        while has_ext {
            let ext: u8 = self.read(reader).ctx(zctx!())?;

            match iext::eid(ext) {
                ext::QoS::ID => {
                    let (q, ext): (ext::QoS, bool) =
                        self.read_knowing_header(reader, ext).ctx(zctx!())?;
                    ext_qos = Some(q);
                    has_ext = ext;
                }
                ext::QoSLink::ID => {
                    let (q, ext): (ext::QoSLink, bool) =
                        self.read_knowing_header(reader, ext).ctx(zctx!())?;
                    ext_qos_link = Some(q);
                    has_ext = ext;
                }
                ext::Auth::ID => {
                    let (a, ext): (ext::Auth, bool) =
                        self.read_knowing_header(reader, ext).ctx(zctx!())?;
                    ext_auth = Some(a);
                    has_ext = ext;
                }
                ext::MultiLink::ID => {
                    let (a, ext): (ext::MultiLink, bool) =
                        self.read_knowing_header(reader, ext).ctx(zctx!())?;
                    ext_mlink = Some(a);
                    has_ext = ext;
                }
                ext::LowLatency::ID => {
                    let (q, ext): (ext::LowLatency, bool) =
                        self.read_knowing_header(reader, ext).ctx(zctx!())?;
                    ext_lowlatency = Some(q);
                    has_ext = ext;
                }
                ext::Compression::ID => {
                    let (q, ext): (ext::Compression, bool) =
                        self.read_knowing_header(reader, ext).ctx(zctx!())?;
                    ext_compression = Some(q);
                    has_ext = ext;
                }
                ext::Patch::ID => {
                    let (p, ext): (ext::PatchType, bool) =
                        self.read_knowing_header(reader, ext).ctx(zctx!())?;
                    ext_patch = p;
                    has_ext = ext;
                }
                _ => {
                    has_ext = extension::skip(reader, "InitSyn", ext)?;
                }
            }
        }

        Ok(InitSyn {
            version,
            whatami,
            zid,
            resolution,
            batch_size,
            ext_qos,
            ext_qos_link,
            ext_auth,
            ext_mlink,
            ext_lowlatency,
            ext_compression,
            ext_patch,
        })
    }

    fn read(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
    ) -> zenoh_result::ZResult<InitSyn<'a>> {
        let header = self.read(reader).ctx(zctx!())?;
        self.read_knowing_header(reader, header).ctx(zctx!())
    }
}

impl<'a> WCodec<'a, &InitAck<'_>> for Zenoh080 {
    fn write(
        &self,
        message: &InitAck<'_>,
        writer: &mut zenoh_buffer::ZBufWriter<'a>,
    ) -> zenoh_result::ZResult<()> {
        let InitAck {
            version,
            whatami,
            zid,
            resolution,
            batch_size,
            cookie,
            ext_qos,
            ext_qos_link,
            ext_auth,
            ext_mlink,
            ext_lowlatency,
            ext_compression,
            ext_patch,
        } = message;

        // Header
        let mut header = id::INIT | flag::A;
        if resolution != &Resolution::default() || batch_size != &batch_size::UNICAST {
            header |= flag::S;
        }
        let mut n_exts = (ext_qos.is_some() as u8)
            + (ext_qos_link.is_some() as u8)
            + (ext_auth.is_some() as u8)
            + (ext_mlink.is_some() as u8)
            + (ext_lowlatency.is_some() as u8)
            + (ext_compression.is_some() as u8)
            + (*ext_patch != ext::PatchType::NONE) as u8;

        if n_exts != 0 {
            header |= flag::Z;
        }
        self.write(header, writer).ctx(zctx!())?;
        self.write(*version, writer).ctx(zctx!())?;

        let whatami: u8 = match whatami {
            WhatAmI::Router => 0b00,
            WhatAmI::Peer => 0b01,
            WhatAmI::Client => 0b10,
        };
        let flags: u8 = ((zid.size() as u8 - 1) << 4) | whatami;
        self.write(flags, writer).ctx(zctx!())?;
        self.write_without_length(zid, writer).ctx(zctx!())?;

        if imsg::has_flag(header, flag::S) {
            self.write(resolution.as_u8(), writer).ctx(zctx!())?;
            self.write(batch_size.to_le_bytes(), writer).ctx(zctx!())?;
        }

        self.write(cookie, writer).ctx(zctx!())?;

        if let Some(qos) = ext_qos.as_ref() {
            n_exts -= 1;
            self.write((qos, n_exts != 0), writer).ctx(zctx!())?;
        }

        if let Some(qos_link) = ext_qos_link.as_ref() {
            n_exts -= 1;
            self.write((qos_link, n_exts != 0), writer).ctx(zctx!())?;
        }

        if let Some(auth) = ext_auth.as_ref() {
            n_exts -= 1;
            self.write((auth, n_exts != 0), writer).ctx(zctx!())?;
        }

        if let Some(mlink) = ext_mlink.as_ref() {
            n_exts -= 1;
            self.write((mlink, n_exts != 0), writer).ctx(zctx!())?;
        }

        if let Some(lowlatency) = ext_lowlatency.as_ref() {
            n_exts -= 1;
            self.write((lowlatency, n_exts != 0), writer).ctx(zctx!())?;
        }

        if let Some(compression) = ext_compression.as_ref() {
            n_exts -= 1;
            self.write((compression, n_exts != 0), writer)
                .ctx(zctx!())?;
        }

        if *ext_patch != ext::PatchType::NONE {
            n_exts -= 1;
            self.write((*ext_patch, n_exts != 0), writer).ctx(zctx!())?;
        }

        Ok(())
    }
}

impl<'a> RCodec<'a, InitAck<'a>> for Zenoh080 {
    fn read_knowing_header(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
        header: u8,
    ) -> zenoh_result::ZResult<InitAck<'a>> {
        if imsg::mid(header) != id::INIT || !imsg::has_flag(header, flag::A) {
            zbail!(ZE::ReadFailure);
        }

        let version: u8 = self.read(reader).ctx(zctx!())?;
        let flags: u8 = self.read(reader).ctx(zctx!())?;

        let whatami = match flags & 0b11 {
            0b00 => WhatAmI::Router,
            0b01 => WhatAmI::Peer,
            0b10 => WhatAmI::Client,
            _ => zbail!(ZE::InvalidBits),
        };

        let length = 1 + ((flags >> 4) as usize);
        let zid: ZenohIdProto = self.read_knowing_length(reader, length)?;
        let mut resolution = Resolution::default();
        let mut batch_size = batch_size::UNICAST.to_le_bytes();

        if imsg::has_flag(header, flag::S) {
            let flags: u8 = self.read(reader).ctx(zctx!())?;
            resolution = Resolution::from(flags & 0b00111111);
            batch_size = self.read(reader).ctx(zctx!())?;
        }

        let batch_size = BatchSize::from_le_bytes(batch_size);
        let cookie: ZBuf<'a> = self.read(reader).ctx(zctx!())?;

        let mut ext_qos = None;
        let mut ext_qos_link = None;
        let mut ext_auth = None;
        let mut ext_mlink = None;
        let mut ext_lowlatency = None;
        let mut ext_compression = None;
        let mut ext_patch = ext::PatchType::NONE;

        let mut has_ext = imsg::has_flag(header, flag::Z);

        while has_ext {
            let ext: u8 = self.read(reader).ctx(zctx!())?;
            match iext::eid(ext) {
                ext::QoS::ID => {
                    let (q, ext): (ext::QoS, bool) =
                        self.read_knowing_header(reader, ext).ctx(zctx!())?;
                    ext_qos = Some(q);
                    has_ext = ext;
                }
                ext::QoSLink::ID => {
                    let (q, ext): (ext::QoSLink, bool) =
                        self.read_knowing_header(reader, ext).ctx(zctx!())?;
                    ext_qos_link = Some(q);
                    has_ext = ext;
                }
                ext::Auth::ID => {
                    let (a, ext): (ext::Auth, bool) =
                        self.read_knowing_header(reader, ext).ctx(zctx!())?;
                    ext_auth = Some(a);
                    has_ext = ext;
                }
                ext::MultiLink::ID => {
                    let (a, ext): (ext::MultiLink, bool) =
                        self.read_knowing_header(reader, ext).ctx(zctx!())?;
                    ext_mlink = Some(a);
                    has_ext = ext;
                }
                ext::LowLatency::ID => {
                    let (q, ext): (ext::LowLatency, bool) =
                        self.read_knowing_header(reader, ext).ctx(zctx!())?;
                    ext_lowlatency = Some(q);
                    has_ext = ext;
                }
                ext::Compression::ID => {
                    let (q, ext): (ext::Compression, bool) =
                        self.read_knowing_header(reader, ext).ctx(zctx!())?;
                    ext_compression = Some(q);
                    has_ext = ext;
                }
                ext::Patch::ID => {
                    let (p, ext): (ext::PatchType, bool) =
                        self.read_knowing_header(reader, ext).ctx(zctx!())?;
                    ext_patch = p;
                    has_ext = ext;
                }
                _ => {
                    has_ext = extension::skip(reader, "InitAck", ext)?;
                }
            }
        }

        Ok(InitAck {
            version,
            whatami,
            zid,
            resolution,
            batch_size,
            cookie,
            ext_qos,
            ext_qos_link,
            ext_auth,
            ext_mlink,
            ext_lowlatency,
            ext_compression,
            ext_patch,
        })
    }

    fn read(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
    ) -> zenoh_result::ZResult<InitAck<'a>> {
        let header = self.read(reader).ctx(zctx!())?;
        self.read_knowing_header(reader, header).ctx(zctx!())
    }
}
