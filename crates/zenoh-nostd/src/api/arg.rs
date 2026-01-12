use std::marker::PhantomData;

use crate::{
    Query, ZConfig,
    api::{Response, Sample},
};

pub trait ZArg {
    type Of<'a>
    where
        Self: 'a;
}

pub struct ResponseRef;
pub struct SampleRef;
pub struct QueryRef<'res, Config>(PhantomData<&'res Config>);

impl ZArg for ResponseRef {
    type Of<'a> = &'a Response<'a>;
}

impl ZArg for SampleRef {
    type Of<'a> = &'a Sample<'a>;
}

impl<'res, Config> ZArg for QueryRef<'res, Config>
where
    Config: ZConfig,
{
    type Of<'a>
        = &'a Query<'a, 'res, Config>
    where
        Config: 'a,
        'res: 'a;
}
