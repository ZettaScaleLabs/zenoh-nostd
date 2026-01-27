use core::marker::PhantomData;

use crate::{api::query::QueryableQuery, config::ZSessionConfig};

use super::{response::*, sample::*};

pub trait ZArg {
    type Of<'a>
    where
        Self: 'a;
}

pub struct GetResponseRef;
pub struct SampleRef;
// pub struct QueryableQueryRef<Config>(PhantomData<Config>);

impl ZArg for GetResponseRef {
    type Of<'a> = &'a GetResponse<'a>;
}

impl ZArg for SampleRef {
    type Of<'a> = &'a Sample<'a>;
}

// impl<Config> ZArg for QueryableQueryRef<Config>
// where
//     Config: ZSessionConfig,
// {
//     type Of<'a>
//         = &'a QueryableQuery<'a, 'static, 'static, 'static, Config>
//     where
//         Self: 'a;
// }
