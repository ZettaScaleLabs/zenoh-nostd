use zenoh_proto::keyexpr;

#[derive(Debug)]
pub struct Sample<'a> {
    ke: &'a keyexpr,
    payload: &'a [u8],
}

impl<'a> Sample<'a> {
    pub fn new(ke: &'a keyexpr, payload: &'a [u8]) -> Self {
        Self { ke, payload }
    }

    pub fn keyexpr(&self) -> &keyexpr {
        self.ke
    }

    pub fn payload(&self) -> &[u8] {
        self.payload
    }
}
