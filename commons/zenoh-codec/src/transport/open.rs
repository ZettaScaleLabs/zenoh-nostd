use core::time::Duration;

use zenoh_buffer::ZBuf;
use zenoh_protocol::{
    common::{extension::iext, imsg},
    transport::{
        id,
        open::{ext, flag, OpenAck, OpenSyn},
        TransportSn,
    },
};
use zenoh_result::{zbail, zctx, WithContext, ZE};

use crate::{common::extension, RCodec, WCodec, ZCodec};

impl<'a> WCodec<'a, &OpenSyn<'_>> for ZCodec {
    fn write(
        &self,
        message: &OpenSyn<'_>,
        writer: &mut zenoh_buffer::ZBufWriter<'a>,
    ) -> zenoh_result::ZResult<()> {
        let OpenSyn {
            lease,
            initial_sn,
            cookie,
            ext_qos,
            ext_auth,
            ext_mlink,
            ext_lowlatency,
            ext_compression,
        } = message;

        // Header
        let mut header = id::OPEN;
        if lease.as_millis() % 1_000 == 0 {
            header |= flag::T;
        }
        let mut n_exts = (ext_qos.is_some() as u8)
            + (ext_auth.is_some() as u8)
            + (ext_mlink.is_some() as u8)
            + (ext_lowlatency.is_some() as u8)
            + (ext_compression.is_some() as u8);

        if n_exts != 0 {
            header |= flag::Z;
        }

        self.write(header, writer).ctx(zctx!())?;

        if imsg::has_flag(header, flag::T) {
            self.write(lease.as_secs() as u32, writer).ctx(zctx!())?;
        } else {
            self.write(lease.as_millis() as u32, writer).ctx(zctx!())?;
        }

        self.write(*initial_sn, writer).ctx(zctx!())?;
        self.write(cookie, writer).ctx(zctx!())?;

        if let Some(qos) = ext_qos.as_ref() {
            n_exts -= 1;
            self.write((qos, n_exts != 0), writer).ctx(zctx!())?;
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

        Ok(())
    }
}

impl<'a> RCodec<'a, OpenSyn<'a>> for ZCodec {
    fn read_knowing_header(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
        header: u8,
    ) -> zenoh_result::ZResult<OpenSyn<'a>> {
        if imsg::mid(header) != id::OPEN || imsg::has_flag(header, flag::A) {
            zbail!(ZE::InvalidBits);
        }

        let lease: u64 = self.read(reader).ctx(zctx!())?;
        let lease = if imsg::has_flag(header, flag::T) {
            Duration::from_secs(lease)
        } else {
            Duration::from_millis(lease)
        };
        let initial_sn: TransportSn = self.read(reader).ctx(zctx!())?;
        let cookie: ZBuf<'_> = self.read(reader).ctx(zctx!())?;

        let mut ext_qos = None;
        let mut ext_auth = None;
        let mut ext_mlink = None;
        let mut ext_lowlatency = None;
        let mut ext_compression = None;

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
                ext::Auth::ID => {
                    let (a, ext): (ext::Auth, bool) =
                        self.read_knowing_header(reader, ext).ctx(zctx!())?;
                    ext_auth = Some(a);
                    has_ext = ext;
                }
                ext::MultiLinkSyn::ID => {
                    let (a, ext): (ext::MultiLinkSyn, bool) =
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
                _ => {
                    has_ext = extension::skip(reader, "OpenSyn", ext).ctx(zctx!())?;
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

    fn read(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
    ) -> zenoh_result::ZResult<OpenSyn<'a>> {
        let header = self.read(reader).ctx(zctx!())?;
        self.read_knowing_header(reader, header).ctx(zctx!())
    }
}

impl<'a> WCodec<'a, &OpenAck<'_>> for ZCodec {
    fn write(
        &self,
        message: &OpenAck<'_>,
        writer: &mut zenoh_buffer::ZBufWriter<'a>,
    ) -> zenoh_result::ZResult<()> {
        let OpenAck {
            lease,
            initial_sn,
            ext_qos,
            ext_auth,
            ext_mlink,
            ext_lowlatency,
            ext_compression,
        } = message;

        let mut header = id::OPEN;
        header |= flag::A;

        if lease.subsec_nanos() == 0 {
            header |= flag::T;
        }

        let mut n_exts = (ext_qos.is_some() as u8)
            + (ext_auth.is_some() as u8)
            + (ext_mlink.is_some() as u8)
            + (ext_lowlatency.is_some() as u8)
            + (ext_compression.is_some() as u8);

        if n_exts != 0 {
            header |= flag::Z;
        }

        self.write(header, writer).ctx(zctx!())?;

        if imsg::has_flag(header, flag::T) {
            self.write(lease.as_secs() as u64, writer).ctx(zctx!())?;
        } else {
            self.write(lease.as_millis() as u64, writer).ctx(zctx!())?;
        }

        self.write(*initial_sn, writer).ctx(zctx!())?;

        if let Some(qos) = ext_qos.as_ref() {
            n_exts -= 1;
            self.write((qos, n_exts != 0), writer).ctx(zctx!())?;
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

        Ok(())
    }
}

impl<'a> RCodec<'a, OpenAck<'a>> for ZCodec {
    fn read_knowing_header(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
        header: u8,
    ) -> zenoh_result::ZResult<OpenAck<'a>> {
        if imsg::mid(header) != id::OPEN || !imsg::has_flag(header, flag::A) {
            zbail!(ZE::InvalidBits);
        }

        // Body
        let lease: u64 = self.read(reader).ctx(zctx!())?;
        let lease = if imsg::has_flag(header, flag::T) {
            Duration::from_secs(lease)
        } else {
            Duration::from_millis(lease)
        };
        let initial_sn: TransportSn = self.read(reader).ctx(zctx!())?;

        // Extensions
        let mut ext_qos = None;
        let mut ext_auth = None;
        let mut ext_mlink = None;
        let mut ext_lowlatency = None;
        let mut ext_compression = None;

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
                ext::Auth::ID => {
                    let (a, ext): (ext::Auth, bool) =
                        self.read_knowing_header(reader, ext).ctx(zctx!())?;
                    ext_auth = Some(a);
                    has_ext = ext;
                }
                ext::MultiLinkAck::ID => {
                    let (a, ext): (ext::MultiLinkAck, bool) =
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
                _ => {
                    has_ext = extension::skip(reader, "OpenAck", ext)?;
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

    fn read(
        &self,
        reader: &mut zenoh_buffer::ZBufReader<'a>,
    ) -> zenoh_result::ZResult<OpenAck<'a>> {
        let header = self.read(reader).ctx(zctx!())?;
        self.read_knowing_header(reader, header).ctx(zctx!())
    }
}
