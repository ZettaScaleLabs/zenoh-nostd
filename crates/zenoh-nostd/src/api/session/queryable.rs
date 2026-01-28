use dyn_utils::storage::RawOrBox;

use crate::api::{arg::QueryableQueryRef, callbacks::FixedCapacityCallbacks};

pub type FixedCapacityQueryableCallbacks<
    'a,
    Config,
    const CAPACITY: usize,
    Callback = RawOrBox<16>,
    Future = RawOrBox<128>,
> = FixedCapacityCallbacks<'a, QueryableQueryRef<'a, Config>, CAPACITY, Callback, Future>;
