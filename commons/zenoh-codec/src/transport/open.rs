use core::time::Duration;

use zenoh_buffers::{
    reader::Reader,
    writer::Writer,
    zslice::{ZSlice, ZSliceLen},
};
use zenoh_protocol::{
    common::{iext, imsg},
    transport::{
        id,
        open::{ext, flag, OpenAck, OpenSyn},
        TransportSn,
    },
};
use zenoh_result::{bail, ZError, ZResult, ZE};

use crate::{common::extension, RCodec, WCodec, Zenoh080, Zenoh080Header};

// OpenSyn
impl<W> WCodec<&OpenSyn, &mut W> for Zenoh080
where
    W: Writer,
{
    type Output = ZResult<()>;

    fn write(self, writer: &mut W, x: &OpenSyn) -> Self::Output {
        let OpenSyn {
            lease,
            initial_sn,
            cookie,
            ext_qos,
            ext_auth,
            ext_mlink,
            ext_lowlatency,
            ext_compression,
        } = x;

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
        self.write(&mut *writer, header)?;

        // Body
        if imsg::has_flag(header, flag::T) {
            self.write(&mut *writer, lease.as_secs())?;
        } else {
            self.write(&mut *writer, lease.as_millis() as u64)?;
        }
        self.write(&mut *writer, initial_sn)?;
        self.write(&mut *writer, cookie)?;

        // Extensions
        if let Some(qos) = ext_qos.as_ref() {
            n_exts -= 1;
            self.write(&mut *writer, (qos, n_exts != 0))?;
        }
        if let Some(auth) = ext_auth.as_ref() {
            n_exts -= 1;
            self.write(&mut *writer, (auth, n_exts != 0))?;
        }
        if let Some(mlink) = ext_mlink.as_ref() {
            n_exts -= 1;
            self.write(&mut *writer, (mlink, n_exts != 0))?;
        }
        if let Some(lowlatency) = ext_lowlatency.as_ref() {
            n_exts -= 1;
            self.write(&mut *writer, (lowlatency, n_exts != 0))?;
        }
        if let Some(compression) = ext_compression.as_ref() {
            n_exts -= 1;
            self.write(&mut *writer, (compression, n_exts != 0))?;
        }

        Ok(())
    }
}

impl<R, const L: usize> RCodec<OpenSyn, &mut R> for (Zenoh080, ZSliceLen<L>)
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<OpenSyn> {
        let header: u8 = self.0.read(&mut *reader)?;
        let codec = (Zenoh080Header::new(header), ZSliceLen::<L>);
        codec.read(reader)
    }
}

impl<R, const L: usize> RCodec<OpenSyn, &mut R> for (Zenoh080Header, ZSliceLen<L>)
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<OpenSyn> {
        if imsg::mid(self.0.header) != id::OPEN || imsg::has_flag(self.0.header, flag::A) {
            bail!(ZE::DidntRead);
        }

        // Body
        let lease: u64 = self.0.codec.read(&mut *reader)?;
        let lease = if imsg::has_flag(self.0.header, flag::T) {
            Duration::from_secs(lease)
        } else {
            Duration::from_millis(lease)
        };
        let initial_sn: TransportSn = self.0.codec.read(&mut *reader)?;
        let cookie: ZSlice = (self.0.codec, ZSliceLen::<L>).read(&mut *reader)?;

        // Extensions
        let mut ext_qos = None;
        let mut ext_auth = None;
        let mut ext_mlink = None;
        let mut ext_lowlatency = None;
        let mut ext_compression = None;

        let mut has_ext = imsg::has_flag(self.0.header, flag::Z);
        while has_ext {
            let ext: u8 = self.0.codec.read(&mut *reader)?;
            let eodec = Zenoh080Header::new(ext);
            match iext::eid(ext) {
                ext::QoS::ID => {
                    let (q, ext): (ext::QoS, bool) = eodec.read(&mut *reader)?;
                    ext_qos = Some(q);
                    has_ext = ext;
                }
                ext::Auth::ID => {
                    let (a, ext): (ext::Auth, bool) = eodec.read(&mut *reader)?;
                    ext_auth = Some(a);
                    has_ext = ext;
                }
                ext::MultiLinkSyn::ID => {
                    let (a, ext): (ext::MultiLinkSyn, bool) = eodec.read(&mut *reader)?;
                    ext_mlink = Some(a);
                    has_ext = ext;
                }
                ext::LowLatency::ID => {
                    let (q, ext): (ext::LowLatency, bool) = eodec.read(&mut *reader)?;
                    ext_lowlatency = Some(q);
                    has_ext = ext;
                }
                ext::Compression::ID => {
                    let (q, ext): (ext::Compression, bool) = eodec.read(&mut *reader)?;
                    ext_compression = Some(q);
                    has_ext = ext;
                }
                _ => {
                    has_ext = extension::skip::<_, 1, 32>(reader, "OpenSyn", ext)?;
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
}

// OpenAck
impl<W> WCodec<&OpenAck, &mut W> for Zenoh080
where
    W: Writer,
{
    type Output = ZResult<()>;

    fn write(self, writer: &mut W, x: &OpenAck) -> Self::Output {
        let OpenAck {
            lease,
            initial_sn,
            ext_qos,
            ext_auth,
            ext_mlink,
            ext_lowlatency,
            ext_compression,
        } = x;

        // Header
        let mut header = id::OPEN;
        header |= flag::A;
        // Verify that the timeout is expressed in seconds, i.e. subsec part is 0.
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
        self.write(&mut *writer, header)?;

        // Body
        if imsg::has_flag(header, flag::T) {
            self.write(&mut *writer, lease.as_secs())?;
        } else {
            self.write(&mut *writer, lease.as_millis() as u64)?;
        }
        self.write(&mut *writer, initial_sn)?;

        // Extensions
        if let Some(qos) = ext_qos.as_ref() {
            n_exts -= 1;
            self.write(&mut *writer, (qos, n_exts != 0))?;
        }
        if let Some(auth) = ext_auth.as_ref() {
            n_exts -= 1;
            self.write(&mut *writer, (auth, n_exts != 0))?;
        }
        if let Some(mlink) = ext_mlink.as_ref() {
            n_exts -= 1;
            self.write(&mut *writer, (mlink, n_exts != 0))?;
        }
        if let Some(lowlatency) = ext_lowlatency.as_ref() {
            n_exts -= 1;
            self.write(&mut *writer, (lowlatency, n_exts != 0))?;
        }
        if let Some(compression) = ext_compression.as_ref() {
            n_exts -= 1;
            self.write(&mut *writer, (compression, n_exts != 0))?;
        }

        Ok(())
    }
}

impl<R> RCodec<OpenAck, &mut R> for Zenoh080
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<OpenAck> {
        let header: u8 = self.read(&mut *reader)?;
        let codec = Zenoh080Header::new(header);
        codec.read(reader)
    }
}

impl<R> RCodec<OpenAck, &mut R> for Zenoh080Header
where
    R: Reader,
{
    type Error = ZError;

    fn read(self, reader: &mut R) -> ZResult<OpenAck> {
        if imsg::mid(self.header) != id::OPEN || !imsg::has_flag(self.header, flag::A) {
            bail!(ZE::DidntRead);
        }

        // Body
        let lease: u64 = self.codec.read(&mut *reader)?;
        let lease = if imsg::has_flag(self.header, flag::T) {
            Duration::from_secs(lease)
        } else {
            Duration::from_millis(lease)
        };
        let initial_sn: TransportSn = self.codec.read(&mut *reader)?;

        // Extensions
        let mut ext_qos = None;
        let mut ext_auth = None;
        let mut ext_mlink = None;
        let mut ext_lowlatency = None;
        let mut ext_compression = None;

        let mut has_ext = imsg::has_flag(self.header, flag::Z);
        while has_ext {
            let ext: u8 = self.codec.read(&mut *reader)?;
            let eodec = Zenoh080Header::new(ext);
            match iext::eid(ext) {
                ext::QoS::ID => {
                    let (q, ext): (ext::QoS, bool) = eodec.read(&mut *reader)?;
                    ext_qos = Some(q);
                    has_ext = ext;
                }
                ext::Auth::ID => {
                    let (a, ext): (ext::Auth, bool) = eodec.read(&mut *reader)?;
                    ext_auth = Some(a);
                    has_ext = ext;
                }
                ext::MultiLinkAck::ID => {
                    let (a, ext): (ext::MultiLinkAck, bool) = eodec.read(&mut *reader)?;
                    ext_mlink = Some(a);
                    has_ext = ext;
                }
                ext::LowLatency::ID => {
                    let (q, ext): (ext::LowLatency, bool) = eodec.read(&mut *reader)?;
                    ext_lowlatency = Some(q);
                    has_ext = ext;
                }
                ext::Compression::ID => {
                    let (q, ext): (ext::Compression, bool) = eodec.read(&mut *reader)?;
                    ext_compression = Some(q);
                    has_ext = ext;
                }
                _ => {
                    has_ext = extension::skip::<_, 1, 32>(reader, "OpenAck", ext)?;
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
}
