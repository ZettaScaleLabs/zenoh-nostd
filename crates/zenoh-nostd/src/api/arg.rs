use crate::api::{Response, Sample};

pub trait ZArg {
    type Of<'a>;
}

pub struct ResponseRef;
pub struct SampleRef;

impl ZArg for ResponseRef {
    type Of<'a> = &'a Response<'a>;
}

impl ZArg for SampleRef {
    type Of<'a> = &'a Sample<'a>;
}
