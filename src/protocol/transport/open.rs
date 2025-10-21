use core::time::Duration;

use crate::{
    protocol::{
        ZCodecError,
        common::{
            extension::{self, iext},
            imsg,
        },
        transport::{TransportSn, id},
        zcodec::{
            decode_u8, decode_u32, decode_u64, decode_zbuf, encode_u8, encode_u32, encode_u64,
            encode_zbuf,
        },
    },
    result::ZResult,
    zbail,
    zbuf::{ZBuf, ZBufReader, ZBufWriter},
};

pub(crate) mod flag {
    pub(crate) const A: u8 = 1 << 5;
    pub(crate) const T: u8 = 1 << 6;
    pub(crate) const Z: u8 = 1 << 7;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OpenSyn<'a> {
    pub(crate) lease: Duration,
    pub(crate) initial_sn: TransportSn,
    pub(crate) cookie: ZBuf<'a>,
    pub(crate) ext_qos: Option<ext::QoS>,
    pub(crate) ext_auth: Option<ext::Auth<'a>>,
    pub(crate) ext_mlink: Option<ext::MultiLinkSyn<'a>>,
    pub(crate) ext_lowlatency: Option<ext::LowLatency>,
    pub(crate) ext_compression: Option<ext::Compression>,
}

impl<'a> OpenSyn<'a> {
    pub(crate) fn encode(&self, writer: &mut ZBufWriter<'_>) -> ZResult<(), ZCodecError> {
        let mut header = id::OPEN;

        if self.lease.as_millis().is_multiple_of(1_000) {
            header |= flag::T;
        }

        let mut n_exts = (self.ext_qos.is_some() as u8)
            + (self.ext_auth.is_some() as u8)
            + (self.ext_mlink.is_some() as u8)
            + (self.ext_lowlatency.is_some() as u8)
            + (self.ext_compression.is_some() as u8);

        if n_exts != 0 {
            header |= flag::Z;
        }

        encode_u8(writer, header)?;

        if imsg::has_flag(header, flag::T) {
            encode_u64(writer, self.lease.as_secs())?;
        } else {
            encode_u64(writer, self.lease.as_millis() as u64)?;
        }

        encode_u32(writer, self.initial_sn)?;
        encode_zbuf(writer, true, self.cookie)?;

        if let Some(qos) = self.ext_qos.as_ref() {
            n_exts -= 1;
            qos.encode(n_exts != 0, writer)?;
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

        Ok(())
    }

    pub(crate) fn decode(header: u8, reader: &mut ZBufReader<'a>) -> ZResult<Self, ZCodecError> {
        if imsg::mid(header) != id::OPEN || imsg::has_flag(header, flag::A) {
            zbail!(ZCodecError::CouldNotRead)
        }

        let lease: u64 = decode_u64(reader)?;
        let lease = if imsg::has_flag(header, flag::T) {
            Duration::from_secs(lease)
        } else {
            Duration::from_millis(lease)
        };
        let initial_sn: TransportSn = decode_u32(reader)?;
        let cookie: ZBuf<'_> = decode_zbuf(reader, None)?;

        let mut ext_qos = None;
        let mut ext_auth = None;
        let mut ext_mlink = None;
        let mut ext_lowlatency = None;
        let mut ext_compression = None;

        let mut has_ext = imsg::has_flag(header, flag::Z);
        while has_ext {
            let ext: u8 = decode_u8(reader)?;
            match iext::eheader(ext) {
                ext::QoS::ID => {
                    let (q, ext) = ext::QoS::decode(ext)?;
                    ext_qos = Some(q);
                    has_ext = ext;
                }
                ext::Auth::ID => {
                    let (a, ext) = ext::Auth::decode(ext, reader)?;
                    ext_auth = Some(a);
                    has_ext = ext;
                }
                ext::MultiLinkSyn::ID => {
                    let (a, ext) = ext::MultiLinkSyn::decode(ext, reader)?;
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
                _ => {
                    has_ext = extension::skip("OpenSyn", ext, reader)?;
                }
            }
        }

        Ok(OpenSyn {
            lease,
            initial_sn,
            cookie,
            ext_qos,
            ext_auth,
            ext_mlink,
            ext_lowlatency,
            ext_compression,
        })
    }

    #[cfg(test)]
    pub(crate) fn rand(zbuf: &mut ZBufWriter<'a>) -> Self {
        use rand::Rng;

        use crate::{
            protocol::common::extension::{ZExtUnit, ZExtZBuf},
            zbuf::BufWriterExt,
        };

        const MIN: usize = 0;
        const MAX: usize = 64;

        let mut rng = rand::thread_rng();

        let lease = if rng.gen_bool(0.5) {
            Duration::from_secs(rng.r#gen())
        } else {
            Duration::from_millis(rng.r#gen())
        };

        let initial_sn: TransportSn = rng.r#gen();
        let cookie = zbuf
            .write_slot_return(rng.gen_range(MIN..=MAX), |b: &mut [u8]| {
                rng.fill(b);
                b.len()
            })
            .unwrap();
        let ext_qos = rng.gen_bool(0.5).then_some(ZExtUnit::rand());
        let ext_auth = rng.gen_bool(0.5).then_some(ZExtZBuf::rand(zbuf));
        let ext_mlink = rng.gen_bool(0.5).then_some(ZExtZBuf::rand(zbuf));
        let ext_lowlatency = rng.gen_bool(0.5).then_some(ZExtUnit::rand());
        let ext_compression = rng.gen_bool(0.5).then_some(ZExtUnit::rand());

        Self {
            lease,
            initial_sn,
            cookie,
            ext_qos,
            ext_auth,
            ext_mlink,
            ext_lowlatency,
            ext_compression,
        }
    }
}

pub(crate) mod ext {

    pub(crate) type QoS = crate::zextunit!(0x1, false);

    pub(crate) type Auth<'a> = crate::zextzbuf!('a, 0x3, false);

    pub(crate) type MultiLinkSyn<'a> = crate::zextzbuf!('a, 0x4, false);
    pub(crate) type MultiLinkAck = crate::zextunit!(0x4, false);

    pub(crate) type LowLatency = crate::zextunit!(0x5, false);

    pub(crate) type Compression = crate::zextunit!(0x6, false);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OpenAck<'a> {
    pub(crate) lease: Duration,
    pub(crate) initial_sn: TransportSn,
    pub(crate) ext_qos: Option<ext::QoS>,
    pub(crate) ext_auth: Option<ext::Auth<'a>>,
    pub(crate) ext_mlink: Option<ext::MultiLinkAck>,
    pub(crate) ext_lowlatency: Option<ext::LowLatency>,
    pub(crate) ext_compression: Option<ext::Compression>,
}

impl<'a> OpenAck<'a> {
    pub(crate) fn encode(&self, writer: &mut ZBufWriter) -> ZResult<(), ZCodecError> {
        let mut header = id::OPEN;
        header |= flag::A;

        if self.lease.subsec_nanos() == 0 {
            header |= flag::T;
        }

        let mut n_exts = (self.ext_qos.is_some() as u8)
            + (self.ext_auth.is_some() as u8)
            + (self.ext_mlink.is_some() as u8)
            + (self.ext_lowlatency.is_some() as u8)
            + (self.ext_compression.is_some() as u8);

        if n_exts != 0 {
            header |= flag::Z;
        }

        encode_u8(writer, header)?;

        if imsg::has_flag(header, flag::T) {
            encode_u64(writer, self.lease.as_secs())?;
        } else {
            encode_u64(writer, self.lease.as_millis() as u64)?;
        }

        encode_u32(writer, self.initial_sn)?;

        if let Some(qos) = self.ext_qos.as_ref() {
            n_exts -= 1;
            qos.encode(n_exts != 0, writer)?;
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

        Ok(())
    }

    pub(crate) fn decode(header: u8, reader: &mut ZBufReader<'a>) -> ZResult<Self, ZCodecError> {
        if imsg::mid(header) != id::OPEN || !imsg::has_flag(header, flag::A) {
            zbail!(ZCodecError::CouldNotRead)
        }

        let lease: u64 = decode_u64(reader)?;
        let lease = if imsg::has_flag(header, flag::T) {
            Duration::from_secs(lease)
        } else {
            Duration::from_millis(lease)
        };
        let initial_sn: TransportSn = decode_u32(reader)?;

        let mut ext_qos = None;
        let mut ext_auth = None;
        let mut ext_mlink = None;
        let mut ext_lowlatency = None;
        let mut ext_compression = None;

        let mut has_ext = imsg::has_flag(header, flag::Z);
        while has_ext {
            let ext: u8 = decode_u8(reader)?;
            match iext::eheader(ext) {
                ext::QoS::ID => {
                    let (q, ext) = ext::QoS::decode(ext)?;
                    ext_qos = Some(q);
                    has_ext = ext;
                }
                ext::Auth::ID => {
                    let (a, ext) = ext::Auth::decode(ext, reader)?;
                    ext_auth = Some(a);
                    has_ext = ext;
                }
                ext::MultiLinkAck::ID => {
                    let (a, ext) = ext::MultiLinkAck::decode(ext)?;
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
                _ => {
                    has_ext = extension::skip("OpenAck", ext, reader)?;
                }
            }
        }

        Ok(OpenAck {
            lease,
            initial_sn,
            ext_qos,
            ext_auth,
            ext_mlink,
            ext_lowlatency,
            ext_compression,
        })
    }

    #[cfg(test)]
    pub(crate) fn rand(zbuf: &mut ZBufWriter<'a>) -> Self {
        use rand::Rng;

        use crate::protocol::common::extension::{ZExtUnit, ZExtZBuf};

        let mut rng = rand::thread_rng();

        let lease = if rng.gen_bool(0.5) {
            Duration::from_secs(rng.r#gen())
        } else {
            Duration::from_millis(rng.r#gen())
        };

        let initial_sn: TransportSn = rng.r#gen();
        let ext_qos = rng.gen_bool(0.5).then_some(ZExtUnit::rand());
        let ext_auth = rng.gen_bool(0.5).then_some(ZExtZBuf::rand(zbuf));
        let ext_mlink = rng.gen_bool(0.5).then_some(ZExtUnit::rand());
        let ext_lowlatency = rng.gen_bool(0.5).then_some(ZExtUnit::rand());
        let ext_compression = rng.gen_bool(0.5).then_some(ZExtUnit::rand());

        Self {
            lease,
            initial_sn,
            ext_qos,
            ext_auth,
            ext_mlink,
            ext_lowlatency,
            ext_compression,
        }
    }
}
