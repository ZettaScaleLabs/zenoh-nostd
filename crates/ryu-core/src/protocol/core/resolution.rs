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
    pub const fn as_u8(&self) -> u8 {
        self.0
    }

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
        let frame_sn = Bits::U32 as u8;
        let request_id = (Bits::U32 as u8) << 2;
        Self(frame_sn | request_id)
    }
}

impl From<u8> for Resolution {
    fn from(v: u8) -> Self {
        Self(v)
    }
}
