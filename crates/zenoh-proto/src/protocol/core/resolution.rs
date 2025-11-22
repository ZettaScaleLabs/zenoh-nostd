use crate::{ZBodyDecode, ZBodyEncode, ZBodyLen};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Bits {
    U8 = 0b0000_0000,
    U16 = 0b0000_0001,
    U32 = 0b0000_0010,
    U64 = 0b0000_0011,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    FrameSN = 0,
    RequestID = 2,
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Resolution(u8);

impl Resolution {
    pub const DEFAULT: Self = {
        let frame_sn = Bits::U32 as u8;
        let request_id = (Bits::U32 as u8) << 2;
        Self(frame_sn | request_id)
    };

    pub const fn get(&self, field: Field) -> Bits {
        let value = (self.0 >> (field as u8)) & 0b11;

        match value {
            0b00 => Bits::U8,
            0b01 => Bits::U16,
            0b10 => Bits::U32,
            0b11 => Bits::U64,
            _ => unreachable!(),
        }
    }

    pub fn set(&mut self, field: Field, bits: Bits) {
        self.0 &= !(0b11 << field as u8);
        self.0 |= (bits as u8) << (field as u8);
    }

    #[cfg(test)]
    pub fn rand() -> Self {
        use rand::Rng;

        let mut rng = rand::thread_rng();
        let v: u8 = rng.r#gen();
        Self(v & 0b0000_1111)
    }
}

impl Default for Resolution {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl From<u8> for Resolution {
    fn from(v: u8) -> Self {
        Self(v)
    }
}

impl ZBodyLen for Resolution {
    fn z_body_len(&self) -> usize {
        <u8 as ZBodyLen>::z_body_len(&self.0)
    }
}

impl ZBodyEncode for Resolution {
    fn z_body_encode(&self, w: &mut crate::ZWriter) -> crate::ZCodecResult<()> {
        <u8 as ZBodyEncode>::z_body_encode(&self.0, w)
    }
}

impl<'a> ZBodyDecode<'a> for Resolution {
    type Ctx = ();

    fn z_body_decode(r: &mut crate::ZReader<'_>, _: ()) -> crate::ZCodecResult<Self> {
        let value = <u8 as crate::ZDecode>::z_decode(r)?;
        Ok(Self(value))
    }
}

crate::derive_zstruct_with_body!(Resolution);
