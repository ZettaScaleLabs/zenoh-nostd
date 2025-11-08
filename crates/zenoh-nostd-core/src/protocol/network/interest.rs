use uhlc::Timestamp;

#[cfg(test)]
use crate::{ZWriter, ZWriterExt};
#[cfg(test)]
use rand::{
    Rng,
    distributions::{Alphanumeric, DistString},
    thread_rng,
};

use crate::{
    ZReaderExt,
    network::{Mapping, NodeId, QoS},
    zbail,
};

// TODO: make it compatible with ZStruct!!! (at least it should be possible to put options and wireexpr in an inner struct)
#[derive(Debug, PartialEq)]
pub struct Interest<'a> {
    pub id: u32,
    pub mode: InterestMode,
    pub options: InterestOptions,

    pub scope: Option<u16>,
    pub mapping: Option<Mapping>,
    pub suffix: Option<&'a str>,

    pub qos: QoS,
    pub timestamp: Option<Timestamp>,
    pub nodeid: NodeId,
}

impl Interest<'_> {
    const HEADER_BASE: u8 = 25u8 << 0u8;
    const HEADER_SLOT_Z: u8 = 0b1 << 7u8;
    pub const ID: u8 = 25u8;

    pub fn options(&self) -> u8 {
        let mut interest = self.options;
        if let Some(suffix) = self.suffix.as_ref() {
            interest.options |= InterestOptions::RESTRICTED.options;
            if !suffix.is_empty() {
                interest.options |= InterestOptions::NAMED.options;
            }
            if let Mapping::Sender = self.mapping.unwrap() {
                interest.options |= InterestOptions::MAPPING.options;
            }
        }
        interest.options
    }
}

impl crate::ZStructEncode for Interest<'_> {
    fn z_len_without_header(&self) -> usize {
        1usize // header
            + <u32 as crate::ZStructEncode>::z_len(&self.id)
            + if self.mode != InterestMode::Final {
                1usize // options header
                    + if let Some(suffix) = &self.suffix {
                        <u16 as crate::ZStructEncode>::z_len(&self.scope.unwrap())
                        + if !suffix.is_empty() {
                            <usize as crate::ZStructEncode>::z_len(&suffix.len())
                                + <&str as crate::ZStructEncode>::z_len(&suffix)
                        } else {
                            0usize
                        }
                    } else {
                        0usize
                    }
            } else {
                0usize
            }
            + if &self.qos != &QoS::DEFAULT {
                crate::zext_len::<_>(&self.qos)
            } else {
                0usize
            }
            + if let Some(inner) = &self.timestamp {
                crate::zext_len::<_>(inner)
            } else {
                0usize
            }
            + if &self.nodeid != &NodeId::DEFAULT {
                crate::zext_len::<_>(&self.nodeid)
            } else {
                0usize
            }
    }
    fn z_encode_without_header(&self, w: &mut crate::ZWriter) -> crate::ZCodecResult<()> {
        let mut header: u8 = Self::HEADER_BASE;
        let mut n_exts = 0;
        if &self.qos != &QoS::DEFAULT {
            n_exts += 1;
        }
        if self.timestamp.is_some() {
            n_exts += 1;
        }
        if &self.nodeid != &NodeId::DEFAULT {
            n_exts += 1;
        }
        if n_exts > 0 {
            header |= Self::HEADER_SLOT_Z;
        }

        header |= match self.mode {
            InterestMode::Final => 0b00,
            InterestMode::Current => 0b01,
            InterestMode::Future => 0b10,
            InterestMode::CurrentFuture => 0b11,
        } << 5;

        <u8 as crate::ZStructEncode>::z_encode(&header, w)?;
        <u32 as crate::ZStructEncode>::z_encode(&self.id, w)?;
        if self.mode != InterestMode::Final {
            <u8 as crate::ZStructEncode>::z_encode(&self.options(), w)?;
            if let Some(suffix) = &self.suffix {
                <u16 as crate::ZStructEncode>::z_encode(&self.scope.unwrap(), w)?;
                if !suffix.is_empty() {
                    <usize as crate::ZStructEncode>::z_encode(&suffix.len(), w)?;
                    <&str as crate::ZStructEncode>::z_encode(suffix, w)?;
                }
            }
        }

        if &self.qos != &QoS::DEFAULT {
            n_exts -= 1;
            crate::zext_encode::<_, 0x1, false>(&self.qos, w, n_exts != 0)?;
        }
        if let Some(inner) = &self.timestamp {
            n_exts -= 1;
            crate::zext_encode::<_, 0x2, false>(inner, w, n_exts != 0)?;
        }
        if &self.nodeid != &NodeId::DEFAULT {
            n_exts -= 1;
            crate::zext_encode::<_, 0x3, true>(&self.nodeid, w, n_exts != 0)?;
        }
        Ok(())
    }
}
impl<'a> crate::ZStructDecode<'a> for Interest<'a> {
    fn z_decode_with_header(r: &mut crate::ZReader<'a>, _: u8) -> crate::ZCodecResult<Self> {
        let header: u8 = <u8 as crate::ZStructDecode>::z_decode(r)?;

        let id = <u32 as crate::ZStructDecode>::z_decode(r)?;
        let mode = match (header >> 5) & 0b11 {
            0b00 => InterestMode::Final,
            0b01 => InterestMode::Current,
            0b10 => InterestMode::Future,
            0b11 => InterestMode::CurrentFuture,
            _ => zbail!(crate::ZCodecError::CouldNotParse),
        };
        let mut options = InterestOptions::empty();

        let mut scope = None;
        let mut mapping = None;
        let mut suffix = None;
        if mode != InterestMode::Final {
            let options_byte: u8 = <u8 as crate::ZStructDecode>::z_decode(r)?;
            options = InterestOptions::options(options_byte);

            if options.restricted() {
                let scope_u16: u16 = <u16 as crate::ZStructDecode>::z_decode(r)?;
                scope = Some(scope_u16);

                if options.named() {
                    let len = <usize as crate::ZStructDecode>::z_decode(r)?;
                    let suffix_str: &str =
                        <&str as crate::ZStructDecode>::z_decode(&mut r.sub(len)?)?;
                    suffix = Some(suffix_str);
                } else {
                    suffix = Some("")
                }

                if options.mapping() {
                    mapping = Some(Mapping::Sender);
                } else {
                    mapping = Some(Mapping::Receiver);
                }
            }
        }

        let mut has_ext: bool = header & Self::HEADER_SLOT_Z != 0;
        let mut qos = QoS::DEFAULT;
        let mut timestamp: _ = None;
        let mut nodeid = NodeId::DEFAULT;
        while has_ext {
            let (ext_id, ext_kind, mandatory, more) = crate::decode_ext_header(r)?;
            has_ext = more;
            match ext_id {
                0x1 => {
                    qos = crate::zext_decode::<_>(r)?;
                }
                0x2 => {
                    timestamp = Some(crate::zext_decode::<_>(r)?);
                }
                0x3 => {
                    nodeid = crate::zext_decode::<_>(r)?;
                }
                _ => {
                    if mandatory {
                        return Err(crate::ZCodecError::UnsupportedMandatoryExtension);
                    }
                    crate::skip_ext(r, ext_kind)?;
                }
            }
        }
        Ok(Self {
            id,
            mode,
            options,
            scope,
            mapping,
            suffix,
            qos,
            timestamp,
            nodeid,
        })
    }
}

impl<'a> Interest<'a> {
    #[cfg(test)]
    pub(crate) fn rand(w: &mut ZWriter<'a>) -> Self {
        let id = thread_rng().r#gen();
        let mode = InterestMode::rand(w);

        let (scope, mapping, suffix) = if mode != InterestMode::Final {
            if thread_rng().gen_bool(0.5) {
                let scope: u16 = thread_rng().r#gen();
                let mapping = Mapping::rand();

                let suffix = if thread_rng().gen_bool(0.5) {
                    let suffix = Alphanumeric
                        .sample_string(&mut thread_rng(), thread_rng().gen_range(1..16));
                    w.write_str(&suffix).unwrap()
                } else {
                    ""
                };
                (Some(scope), Some(mapping), Some(suffix))
            } else {
                (None, None, None)
            }
        } else {
            (None, None, None)
        };

        let qos = if thread_rng().gen_bool(0.5) {
            QoS::rand(w)
        } else {
            QoS::DEFAULT
        };

        let timestamp = thread_rng().gen_bool(0.5).then_some({
            use crate::protocol::core::ZenohIdProto;

            let time = uhlc::NTP64(thread_rng().r#gen());
            let id = uhlc::ID::try_from(ZenohIdProto::default().as_le_bytes()).unwrap();
            Timestamp::new(time, id)
        });

        let nodeid = if thread_rng().gen_bool(0.5) {
            NodeId::rand(w)
        } else {
            NodeId::DEFAULT
        };

        let options = if mode != InterestMode::Final {
            InterestOptions::rand()
        } else {
            InterestOptions::empty()
        };

        Self {
            id,
            mode,
            options,
            scope,
            mapping,
            suffix,
            qos,
            timestamp,
            nodeid,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InterestMode {
    Final,
    Current,
    Future,
    CurrentFuture,
}

impl InterestMode {
    #[cfg(test)]
    pub fn rand(_: &mut ZWriter) -> Self {
        match thread_rng().gen_range(0..4) {
            0 => InterestMode::Final,
            1 => InterestMode::Current,
            2 => InterestMode::Future,
            3 => InterestMode::CurrentFuture,
            _ => unreachable!(),
        }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct InterestOptions {
    options: u8,
}

impl PartialEq for InterestOptions {
    fn eq(&self, other: &Self) -> bool {
        self.keyexprs() == other.keyexprs()
            && self.subscribers() == other.subscribers()
            && self.queryables() == other.queryables()
            && self.tokens() == other.tokens()
            && self.aggregate() == other.aggregate()
    }
}

impl InterestOptions {
    pub const KEYEXPRS: InterestOptions = InterestOptions::options(1);
    pub const SUBSCRIBERS: InterestOptions = InterestOptions::options(1 << 1);
    pub const QUERYABLES: InterestOptions = InterestOptions::options(1 << 2);
    pub const TOKENS: InterestOptions = InterestOptions::options(1 << 3);

    const RESTRICTED: InterestOptions = InterestOptions::options(1 << 4);
    const NAMED: InterestOptions = InterestOptions::options(1 << 5);
    const MAPPING: InterestOptions = InterestOptions::options(1 << 6);

    pub const AGGREGATE: InterestOptions = InterestOptions::options(1 << 7);
    pub const ALL: InterestOptions = InterestOptions::options(
        InterestOptions::KEYEXPRS.options
            | InterestOptions::SUBSCRIBERS.options
            | InterestOptions::QUERYABLES.options
            | InterestOptions::TOKENS.options,
    );

    const fn options(options: u8) -> Self {
        Self { options }
    }

    pub const fn empty() -> Self {
        Self { options: 0 }
    }

    pub const fn keyexprs(&self) -> bool {
        self.options & Self::KEYEXPRS.options != 0
    }

    pub const fn subscribers(&self) -> bool {
        self.options & Self::SUBSCRIBERS.options != 0
    }

    pub const fn queryables(&self) -> bool {
        self.options & Self::QUERYABLES.options != 0
    }

    pub const fn tokens(&self) -> bool {
        self.options & Self::TOKENS.options != 0
    }

    pub const fn restricted(&self) -> bool {
        self.options & Self::RESTRICTED.options != 0
    }

    pub const fn named(&self) -> bool {
        self.options & Self::NAMED.options != 0
    }

    pub const fn mapping(&self) -> bool {
        self.options & Self::MAPPING.options != 0
    }

    pub const fn aggregate(&self) -> bool {
        self.options & Self::AGGREGATE.options != 0
    }
}

impl InterestOptions {
    #[cfg(test)]
    pub fn rand() -> Self {
        let mut s = Self::empty();
        if thread_rng().gen_bool(0.5) {
            s.options |= Self::KEYEXPRS.options;
        }
        if thread_rng().gen_bool(0.5) {
            s.options |= Self::SUBSCRIBERS.options;
        }
        if thread_rng().gen_bool(0.5) {
            s.options |= Self::TOKENS.options;
        }
        if thread_rng().gen_bool(0.5) {
            s.options |= Self::AGGREGATE.options;
        }
        s
    }
}
