use zenoh_proto::keyexpr;

pub struct Sample {
    keyexpr: *const keyexpr,
    payload: *const [u8],
}

impl Sample {
    pub(crate) fn new(keyexpr: &keyexpr, payload: &[u8]) -> Sample {
        Sample { keyexpr, payload }
    }

    pub unsafe fn keyexpr(&self) -> &keyexpr {
        unsafe { &*self.keyexpr }
    }

    pub unsafe fn payload(&self) -> &[u8] {
        unsafe { &*self.payload }
    }
}
