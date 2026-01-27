use crate::{
    // api::{
    //     arg::{GetResponseRef, SampleRef},
    //     callbacks::ZCallbacks,
    // },
    io::{link::ZLinkManager, transport::TransportLinkManager},
};

pub trait ZSessionConfig {
    type Buff: AsMut<[u8]> + AsRef<[u8]> + Clone;
    type LinkManager: ZLinkManager;

    // type GetCallbacks<'res>: ZCallbacks<'res, GetResponseRef>;
    // type SubCallbacks<'res>: ZCallbacks<'res, SampleRef>;
    // type QueryableCallbacks<'res>: ZCallbacks<'res, QueryableQueryRef<'res, Self>>;

    fn transports(&self) -> &TransportLinkManager<Self::LinkManager>;
    fn buff(&self) -> Self::Buff;
}

// pub type FixedCapacityGetCallbacks<
//     'a,
//     const CAPACITY: usize,
//     Callback = RawOrBox<16>,
//     Future = RawOrBox<128>,
// > = FixedCapacityCallbacks<'a, ResponseRef, CAPACITY, Callback, Future>;

// pub type FixedCapacityQueryableCallbacks<
//     'a,
//     Config,
//     const CAPACITY: usize,
//     Callback = RawOrBox<16>,
//     Future = RawOrBox<128>,
// > = FixedCapacityCallbacks<'a, QueryRef<'a, Config>, CAPACITY, Callback, Future>;
