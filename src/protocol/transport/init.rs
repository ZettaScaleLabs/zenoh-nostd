use crate::{
    protocol::{
        ZCodecError,
        common::{
            extension::{self, iext},
            imsg,
        },
        core::{ZenohIdProto, resolution::Resolution, whatami::WhatAmI},
        transport::{BatchSize, batch_size, id},
        zcodec::{decode_array, decode_u8, decode_zbuf, encode_array, encode_u8, encode_zbuf},
    },
    result::ZResult,
    zbail,
    zbuf::{ZBuf, ZBufReader, ZBufWriter},
};

pub(crate) mod flag {
    pub(crate) const A: u8 = 1 << 5;
    pub(crate) const S: u8 = 1 << 6;
    pub(crate) const Z: u8 = 1 << 7;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InitSyn<'a> {
    pub(crate) version: u8,
    pub(crate) whatami: WhatAmI,
    pub(crate) zid: ZenohIdProto,
    pub(crate) resolution: Resolution,
    pub(crate) batch_size: BatchSize,
    pub(crate) ext_qos: Option<ext::QoS>,
    pub(crate) ext_qos_link: Option<ext::QoSLink>,
    pub(crate) ext_auth: Option<ext::Auth<'a>>,
    pub(crate) ext_mlink: Option<ext::MultiLink<'a>>,
    pub(crate) ext_lowlatency: Option<ext::LowLatency>,
    pub(crate) ext_compression: Option<ext::Compression>,
    pub(crate) ext_patch: ext::PatchType,
}

impl<'a> InitSyn<'a> {
    pub(crate) fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
        let mut header = id::INIT;
        if self.resolution != Resolution::default() || self.batch_size != batch_size::UNICAST {
            header |= flag::S;
        }
        let mut n_exts = (self.ext_qos.is_some() as u8)
            + (self.ext_qos_link.is_some() as u8)
            + (self.ext_auth.is_some() as u8)
            + (self.ext_mlink.is_some() as u8)
            + (self.ext_lowlatency.is_some() as u8)
            + (self.ext_compression.is_some() as u8)
            + (self.ext_patch != ext::PatchType::NONE) as u8;

        if n_exts != 0 {
            header |= flag::Z;
        }

        encode_u8(writer, header)?;
        encode_u8(writer, self.version)?;

        let whatami: u8 = match self.whatami {
            WhatAmI::Router => 0b00,
            WhatAmI::Peer => 0b01,
            WhatAmI::Client => 0b10,
        };

        let flags: u8 = ((self.zid.size() as u8 - 1) << 4) | whatami;
        encode_u8(writer, flags)?;
        self.zid.encode(false, writer)?;

        if imsg::has_flag(header, flag::S) {
            encode_u8(writer, self.resolution.as_u8())?;
            encode_array(writer, &self.batch_size.to_le_bytes())?;
        }

        if let Some(qos) = self.ext_qos.as_ref() {
            n_exts -= 1;
            qos.encode(n_exts != 0, writer)?;
        }

        if let Some(qos_link) = self.ext_qos_link.as_ref() {
            n_exts -= 1;
            qos_link.encode(n_exts != 0, writer)?;
        }

        if let Some(auth) = self.ext_auth.as_ref() {
            n_exts -= 1;
            auth.encode(n_exts != 0, writer)?;
        }

        if let Some(mlink) = self.ext_mlink.as_ref() {
            n_exts -= 1;
            mlink.encode(n_exts != 0, writer)?;
        }

        if let Some(lowlatency) = self.ext_lowlatency.as_ref() {
            n_exts -= 1;
            lowlatency.encode(n_exts != 0, writer)?;
        }

        if let Some(compression) = self.ext_compression.as_ref() {
            n_exts -= 1;
            compression.encode(n_exts != 0, writer)?;
        }

        if self.ext_patch != ext::PatchType::NONE {
            n_exts -= 1;
            self.ext_patch.encode(n_exts != 0, writer)?;
        }

        Ok(())
    }

    pub(crate) fn decode(header: u8, reader: &mut ZBufReader<'a>) -> ZResult<Self, ZCodecError> {
        if imsg::mid(header) != id::INIT || imsg::has_flag(header, flag::A) {
            zbail!(ZCodecError::CouldNotRead)
        }

        let version: u8 = decode_u8(reader)?;
        let flags: u8 = decode_u8(reader)?;
        let whatami = match flags & 0b11 {
            0b00 => WhatAmI::Router,
            0b01 => WhatAmI::Peer,
            0b10 => WhatAmI::Client,
            _ => zbail!(ZCodecError::CouldNotParse),
        };

        let length = 1 + ((flags >> 4) as usize);
        let zid: ZenohIdProto = ZenohIdProto::decode(Some(length), reader)?;

        let mut resolution = Resolution::default();
        let mut batch_size = batch_size::UNICAST.to_le_bytes();

        if imsg::has_flag(header, flag::S) {
            let flags: u8 = decode_u8(reader)?;
            resolution = Resolution::from(flags & 0b00111111);
            batch_size = decode_array(reader)?;
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
            let ext: u8 = decode_u8(reader)?;

            match iext::eheader(ext) {
                ext::QoS::ID => {
                    let (q, ext) = ext::QoS::decode(ext)?;
                    ext_qos = Some(q);
                    has_ext = ext;
                }
                ext::QoSLink::ID => {
                    let (q, ext) = ext::QoSLink::decode(ext, reader)?;
                    ext_qos_link = Some(q);
                    has_ext = ext;
                }
                ext::Auth::ID => {
                    let (a, ext) = ext::Auth::decode(ext, reader)?;
                    ext_auth = Some(a);
                    has_ext = ext;
                }
                ext::MultiLink::ID => {
                    let (a, ext) = ext::MultiLink::decode(ext, reader)?;
                    ext_mlink = Some(a);
                    has_ext = ext;
                }
                ext::LowLatency::ID => {
                    let (q, ext) = ext::LowLatency::decode(ext)?;
                    ext_lowlatency = Some(q);
                    has_ext = ext;
                }
                ext::Compression::ID => {
                    let (q, ext) = ext::Compression::decode(ext)?;
                    ext_compression = Some(q);
                    has_ext = ext;
                }
                ext::Patch::ID => {
                    let (p, ext) = ext::PatchType::decode(ext, reader)?;
                    ext_patch = p;
                    has_ext = ext;
                }
                _ => {
                    has_ext = extension::skip("Init Syn", ext, reader)?;
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

    #[cfg(test)]
    pub(crate) fn rand(zbuf: &mut ZBufWriter<'a>) -> Self {
        use rand::Rng;

        use crate::protocol::common::extension::{ZExtUnit, ZExtZ64, ZExtZBuf};

        let mut rng = rand::thread_rng();

        let version: u8 = rng.r#gen();
        let whatami = WhatAmI::rand();
        let zid = ZenohIdProto::default();
        let resolution = Resolution::rand();
        let batch_size: BatchSize = rng.r#gen();
        let ext_qos = rng.gen_bool(0.5).then_some(ZExtUnit::rand());
        let ext_qos_link = rng.gen_bool(0.5).then_some(ZExtZ64::rand());
        let ext_auth = rng.gen_bool(0.5).then_some(ZExtZBuf::rand(zbuf));
        let ext_mlink = rng.gen_bool(0.5).then_some(ZExtZBuf::rand(zbuf));
        let ext_lowlatency = rng.gen_bool(0.5).then_some(ZExtUnit::rand());
        let ext_compression = rng.gen_bool(0.5).then_some(ZExtUnit::rand());
        let ext_patch = ext::PatchType::rand();

        Self {
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
        }
    }
}

pub(crate) mod ext {
    pub(crate) type QoS = crate::zextunit!(0x1, false);
    pub(crate) type QoSLink = crate::zextz64!(0x1, false);

    pub(crate) type Auth<'a> = crate::zextzbuf!('a, 0x3, false);

    pub(crate) type MultiLink<'a> = crate::zextzbuf!('a, 0x4, false);

    pub(crate) type LowLatency = crate::zextunit!(0x5, false);

    pub(crate) type Compression = crate::zextunit!(0x6, false);

    pub(crate) type Patch = crate::zextz64!(0x7, false);
    pub(crate) type PatchType = crate::protocol::transport::ext::PatchType<{ Patch::ID }>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InitAck<'a> {
    pub(crate) version: u8,
    pub(crate) whatami: WhatAmI,
    pub(crate) zid: ZenohIdProto,
    pub(crate) resolution: Resolution,
    pub(crate) batch_size: BatchSize,
    pub(crate) cookie: ZBuf<'a>,
    pub(crate) ext_qos: Option<ext::QoS>,
    pub(crate) ext_qos_link: Option<ext::QoSLink>,
    pub(crate) ext_auth: Option<ext::Auth<'a>>,
    pub(crate) ext_mlink: Option<ext::MultiLink<'a>>,
    pub(crate) ext_lowlatency: Option<ext::LowLatency>,
    pub(crate) ext_compression: Option<ext::Compression>,
    pub(crate) ext_patch: ext::PatchType,
}

impl<'a> InitAck<'a> {
    pub(crate) fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
        let mut header = id::INIT | flag::A;
        if self.resolution != Resolution::default() || self.batch_size != batch_size::UNICAST {
            header |= flag::S;
        }
        let mut n_exts = (self.ext_qos.is_some() as u8)
            + (self.ext_qos_link.is_some() as u8)
            + (self.ext_auth.is_some() as u8)
            + (self.ext_mlink.is_some() as u8)
            + (self.ext_lowlatency.is_some() as u8)
            + (self.ext_compression.is_some() as u8)
            + (self.ext_patch != ext::PatchType::NONE) as u8;

        if n_exts != 0 {
            header |= flag::Z;
        }

        encode_u8(writer, header)?;
        encode_u8(writer, self.version)?;

        let whatami: u8 = match self.whatami {
            WhatAmI::Router => 0b00,
            WhatAmI::Peer => 0b01,
            WhatAmI::Client => 0b10,
        };

        let flags: u8 = ((self.zid.size() as u8 - 1) << 4) | whatami;
        encode_u8(writer, flags)?;
        self.zid.encode(false, writer)?;

        if imsg::has_flag(header, flag::S) {
            encode_u8(writer, self.resolution.as_u8())?;
            encode_array(writer, &self.batch_size.to_le_bytes())?;
        }

        encode_zbuf(writer, true, self.cookie)?;

        if let Some(qos) = self.ext_qos.as_ref() {
            n_exts -= 1;
            qos.encode(n_exts != 0, writer)?;
        }

        if let Some(qos_link) = self.ext_qos_link.as_ref() {
            n_exts -= 1;
            qos_link.encode(n_exts != 0, writer)?;
        }

        if let Some(auth) = self.ext_auth.as_ref() {
            n_exts -= 1;
            auth.encode(n_exts != 0, writer)?;
        }

        if let Some(mlink) = self.ext_mlink.as_ref() {
            n_exts -= 1;
            mlink.encode(n_exts != 0, writer)?;
        }

        if let Some(lowlatency) = self.ext_lowlatency.as_ref() {
            n_exts -= 1;
            lowlatency.encode(n_exts != 0, writer)?;
        }

        if let Some(compression) = self.ext_compression.as_ref() {
            n_exts -= 1;
            compression.encode(n_exts != 0, writer)?;
        }

        if self.ext_patch != ext::PatchType::NONE {
            n_exts -= 1;
            self.ext_patch.encode(n_exts != 0, writer)?;
        }

        Ok(())
    }

    pub(crate) fn decode(header: u8, reader: &mut ZBufReader<'a>) -> ZResult<Self, ZCodecError> {
        if imsg::mid(header) != id::INIT || !imsg::has_flag(header, flag::A) {
            zbail!(ZCodecError::CouldNotRead)
        }

        let version: u8 = decode_u8(reader)?;
        let flags: u8 = decode_u8(reader)?;

        let whatami = match flags & 0b11 {
            0b00 => WhatAmI::Router,
            0b01 => WhatAmI::Peer,
            0b10 => WhatAmI::Client,
            _ => zbail!(ZCodecError::CouldNotParse),
        };

        let length = 1 + ((flags >> 4) as usize);
        let zid: ZenohIdProto = ZenohIdProto::decode(Some(length), reader)?;
        let mut resolution = Resolution::default();
        let mut batch_size = batch_size::UNICAST.to_le_bytes();

        if imsg::has_flag(header, flag::S) {
            let flags: u8 = decode_u8(reader)?;
            resolution = Resolution::from(flags & 0b00111111);
            batch_size = decode_array(reader)?;
        }

        let batch_size = BatchSize::from_le_bytes(batch_size);
        let cookie: ZBuf<'a> = decode_zbuf(reader, None)?;

        let mut ext_qos = None;
        let mut ext_qos_link = None;
        let mut ext_auth = None;
        let mut ext_mlink = None;
        let mut ext_lowlatency = None;
        let mut ext_compression = None;
        let mut ext_patch = ext::PatchType::NONE;

        let mut has_ext = imsg::has_flag(header, flag::Z);

        while has_ext {
            let ext: u8 = decode_u8(reader)?;
            match iext::eheader(ext) {
                ext::QoS::ID => {
                    let (q, ext) = ext::QoS::decode(ext)?;
                    ext_qos = Some(q);
                    has_ext = ext;
                }
                ext::QoSLink::ID => {
                    let (q, ext) = ext::QoSLink::decode(ext, reader)?;
                    ext_qos_link = Some(q);
                    has_ext = ext;
                }
                ext::Auth::ID => {
                    let (a, ext) = ext::Auth::decode(ext, reader)?;
                    ext_auth = Some(a);
                    has_ext = ext;
                }
                ext::MultiLink::ID => {
                    let (a, ext) = ext::MultiLink::decode(ext, reader)?;
                    ext_mlink = Some(a);
                    has_ext = ext;
                }
                ext::LowLatency::ID => {
                    let (q, ext) = ext::LowLatency::decode(ext)?;
                    ext_lowlatency = Some(q);
                    has_ext = ext;
                }
                ext::Compression::ID => {
                    let (q, ext) = ext::Compression::decode(ext)?;
                    ext_compression = Some(q);
                    has_ext = ext;
                }
                ext::Patch::ID => {
                    let (p, ext) = ext::PatchType::decode(ext, reader)?;
                    ext_patch = p;
                    has_ext = ext;
                }
                _ => {
                    has_ext = extension::skip("InitAck", ext, reader)?;
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

    #[cfg(test)]
    pub(crate) fn rand(zbuf: &mut ZBufWriter<'a>) -> Self {
        use rand::Rng;

        use crate::{
            protocol::common::extension::{ZExtUnit, ZExtZ64, ZExtZBuf},
            zbuf::BufWriterExt,
        };

        let mut rng = rand::thread_rng();

        let version: u8 = rng.r#gen();
        let whatami = WhatAmI::rand();
        let zid = ZenohIdProto::default();
        let resolution = if rng.gen_bool(0.5) {
            Resolution::default()
        } else {
            Resolution::rand()
        };
        let batch_size: BatchSize = rng.r#gen();
        let cookie = zbuf
            .write_slot_return(64, |b: &mut [u8]| {
                rng.fill(b);
                b.len()
            })
            .unwrap();
        let ext_qos = rng.gen_bool(0.5).then_some(ZExtUnit::rand());
        let ext_qos_link = rng.gen_bool(0.5).then_some(ZExtZ64::rand());
        let ext_auth = rng.gen_bool(0.5).then_some(ZExtZBuf::rand(zbuf));
        let ext_mlink = rng.gen_bool(0.5).then_some(ZExtZBuf::rand(zbuf));
        let ext_lowlatency = rng.gen_bool(0.5).then_some(ZExtUnit::rand());
        let ext_compression = rng.gen_bool(0.5).then_some(ZExtUnit::rand());
        let ext_patch = ext::PatchType::rand();

        Self {
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
        }
    }
}
