use super::{response::*, sample::*};

pub trait ZArg {
    type Of<'a>
    where
        Self: 'a;
}

pub struct GetResponseRef;
pub struct SampleRef;

impl ZArg for GetResponseRef {
    type Of<'a> = &'a GetResponse<'a>;
}

impl ZArg for SampleRef {
    type Of<'a> = &'a Sample<'a>;
}
