use zenoh_proto::keyexpr;

use crate::api::Sample;

#[derive(Debug)]
pub enum Response<'a> {
    Ok(Sample<'a>),
    Err(Sample<'a>),
}

impl<'a> Response<'a> {
    pub fn ok(ke: &'a keyexpr, payload: &'a [u8]) -> Self {
        Self::Ok(Sample::new(ke, payload))
    }

    pub fn err(ke: &'a keyexpr, payload: &'a [u8]) -> Self {
        Self::Err(Sample::new(ke, payload))
    }
}
