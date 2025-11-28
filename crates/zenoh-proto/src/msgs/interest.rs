use crate::{exts::*, *};

impl InterestInner<'_> {
    const HEADER_SLOT_FULL: u8 = 0b1111_1111;
}
impl Interest<'_> {
    const HEADER_BASE: u8 = 25u8;
    pub const ID: u8 = 25u8;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InterestMode {
    Final,
    Current,
    Future,
    CurrentFuture,
}

#[derive(ZStruct, Debug)]
#[zenoh(header = "A|M|N|R|T|Q|S|K")]
pub struct InterestInner<'a> {
    #[zenoh(header = FULL)]
    pub options: u8,

    #[zenoh(presence = header(R), flatten, shift = 5)]
    pub wire_expr: Option<WireExpr<'a>>,
}

#[derive(ZStruct, Debug, PartialEq)]
#[zenoh(header = "Z|_:2|ID:5=0x19")]
pub struct InterestExt {
    #[zenoh(ext = 0x1, default = QoS::DEFAULT)]
    pub qos: QoS,
    #[zenoh(ext = 0x2)]
    pub timestamp: Option<Timestamp>,
    #[zenoh(ext = 0x3, default = NodeId::DEFAULT, mandatory)]
    pub nodeid: NodeId,
}

#[derive(Debug, PartialEq)]
// #[zenoh(header = "Z|MODE:2|ID:5=0x19")]
pub struct Interest<'a> {
    pub id: u32,
    // #[zenoh(header = MODE)]
    pub mode: InterestMode,

    // #[zenoh(headercond(MODE) != InterestMode::FINAL)]
    pub inner: InterestInner<'a>,

    // #[zenoh(flatten)]
    pub ext: InterestExt,
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct InterestOptions {
    pub options: u8,
}

impl PartialEq for InterestInner<'_> {
    fn eq(&self, other: &Self) -> bool {
        let options = InterestOptions::options(self.options);
        let other_options = InterestOptions::options(other.options);

        options == other_options && self.wire_expr == other.wire_expr
    }
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

    pub const AGGREGATE: InterestOptions = InterestOptions::options(1 << 7);

    const fn options(options: u8) -> Self {
        Self { options }
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

    pub const fn aggregate(&self) -> bool {
        self.options & Self::AGGREGATE.options != 0
    }
}

impl ZHeader for Interest<'_> {
    fn z_header(&self) -> u8 {
        let mut header: u8 = Self::HEADER_BASE;

        header |= match self.mode {
            InterestMode::Final => 0b00,
            InterestMode::Current => 0b01,
            InterestMode::Future => 0b10,
            InterestMode::CurrentFuture => 0b11,
        } << 5;

        header |= <_ as ZHeader>::z_header(&self.ext);

        header
    }
}

impl ZBodyLen for Interest<'_> {
    fn z_body_len(&self) -> usize {
        <u32 as ZLen>::z_len(&self.id)
            + if self.mode != InterestMode::Final {
                <_ as ZLen>::z_len(&self.inner)
            } else {
                0usize
            }
            + self.ext.z_body_len()
    }
}

impl ZBodyEncode for Interest<'_> {
    fn z_body_encode(&self, w: &mut ZWriter) -> crate::ZResult<(), crate::ZCodecError> {
        <u32 as ZBodyEncode>::z_body_encode(&self.id, w)?;

        if self.mode != InterestMode::Final {
            <_ as ZEncode>::z_encode(&self.inner, w)?;
        }

        <_ as ZBodyEncode>::z_body_encode(&self.ext, w)?;

        Ok(())
    }
}

impl<'a> ZBodyDecode<'a> for Interest<'a> {
    type Ctx = u8;

    fn z_body_decode(
        r: &mut crate::ZReader<'a>,
        header: u8,
    ) -> crate::ZResult<Self, crate::ZCodecError> {
        let id = <u32 as ZDecode>::z_decode(r)?;

        let mode = match (header >> 5) & 0b11 {
            0b00 => InterestMode::Final,
            0b01 => InterestMode::Current,
            0b10 => InterestMode::Future,
            0b11 => InterestMode::CurrentFuture,
            _ => zbail!(crate::ZCodecError::CouldNotParseField),
        };

        let inner = if mode != InterestMode::Final {
            <_ as ZDecode>::z_decode(r)?
        } else {
            InterestInner {
                options: 0,
                wire_expr: None,
            }
        };

        let ext = <_ as ZBodyDecode>::z_body_decode(r, header)?;

        Ok(Self {
            id,
            mode,
            inner,
            ext,
        })
    }
}

impl ZLen for Interest<'_> {
    fn z_len(&self) -> usize {
        1usize + <Self as ZBodyLen>::z_body_len(self)
    }
}

impl ZEncode for Interest<'_> {
    fn z_encode(&self, w: &mut ZWriter) -> crate::ZResult<(), crate::ZCodecError> {
        let header = <Self as ZHeader>::z_header(self);
        <u8 as ZEncode>::z_encode(&header, w)?;
        <Self as ZBodyEncode>::z_body_encode(self, w)
    }
}

impl<'a> ZDecode<'a> for Interest<'a> {
    fn z_decode(r: &mut crate::ZReader<'a>) -> crate::ZResult<Self, crate::ZCodecError> {
        let header = <u8 as ZDecode>::z_decode(r)?;
        <Self as ZBodyDecode>::z_body_decode(r, header)
    }
}
