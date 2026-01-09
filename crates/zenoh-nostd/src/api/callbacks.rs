use std::marker::PhantomData;

use dyn_utils::{
    DynObject,
    storage::{RawOrBox, Storage},
};
use embassy_time::Instant;
use heapless::FnvIndexMap;
use zenoh_proto::keyexpr;

use crate::api::arg::{ResponseRef, SampleRef, ZArg};

#[dyn_utils::dyn_trait(trait = ZDynCallback)]
#[dyn_trait(dyn_utils::dyn_object)]
pub trait ZCallback {
    type Arg: ZArg;

    #[dyn_trait(try_sync)]
    fn call(&mut self, arg: <Self::Arg as ZArg>::Of<'_>) -> impl Future<Output = ()>;
}

pub type DynCallback<'a, Callback, Future, Arg> =
    DynObject<dyn ZDynCallback<Future, Arg = Arg> + 'a, Callback>;

pub trait ZCallbacks<'a, Arg>
where
    Arg: ZArg + 'a,
{
    type Callback: Storage;
    type Future: Storage;

    fn empty() -> Self;

    fn insert(
        &mut self,
        id: u32,
        ke: &'static keyexpr,
        timedout: Option<Instant>,
        callback: DynCallback<'a, Self::Callback, Self::Future, Arg>,
    ) -> core::result::Result<(), crate::CollectionError>;

    fn drop_timedout(&mut self);
    fn get(&mut self, id: u32) -> Option<&mut DynCallback<'a, Self::Callback, Self::Future, Arg>>;

    fn remove(&mut self, id: u32) -> core::result::Result<(), crate::CollectionError>;

    fn intersects<'r>(
        &'r mut self,
        ke: &keyexpr,
    ) -> impl Iterator<Item = &'r mut DynCallback<'a, Self::Callback, Self::Future, Arg>>
    where
        DynCallback<'a, Self::Callback, Self::Future, Arg>: 'r;
}

pub struct FixedCapacityCallbacks<
    'a,
    Arg: ZArg,
    const CAPACITY: usize,
    Callback: Storage,
    Future: Storage,
> {
    keyexprs: FnvIndexMap<u32, &'static keyexpr, CAPACITY>,
    callbacks:
        FnvIndexMap<(u32, &'static keyexpr), DynCallback<'a, Callback, Future, Arg>, CAPACITY>,
    timedouts: FnvIndexMap<u32, Instant, CAPACITY>,
}

impl<'a, Arg: ZArg + 'a, const CAPACITY: usize, Callback: Storage, Future: Storage>
    ZCallbacks<'a, Arg> for FixedCapacityCallbacks<'a, Arg, CAPACITY, Callback, Future>
{
    type Callback = Callback;
    type Future = Future;

    fn empty() -> Self {
        Self {
            keyexprs: FnvIndexMap::new(),
            callbacks: FnvIndexMap::new(),
            timedouts: FnvIndexMap::new(),
        }
    }

    fn insert(
        &mut self,
        id: u32,
        ke: &'static keyexpr,
        timedout: Option<Instant>,
        callback: DynCallback<'a, Callback, Future, Arg>,
    ) -> core::result::Result<(), crate::CollectionError> {
        if self.keyexprs.contains_key(&id) {
            return Err(crate::CollectionError::KeyAlreadyExists);
        }

        if self.callbacks.contains_key(&(id, ke)) {
            return Err(crate::CollectionError::KeyAlreadyExists);
        }

        if self.timedouts.contains_key(&id) {
            return Err(crate::CollectionError::KeyAlreadyExists);
        }

        self.keyexprs
            .insert(id, ke)
            .map_err(|_| crate::CollectionError::CollectionIsFull)?;

        self.callbacks
            .insert((id, ke), callback)
            .map_err(|_| crate::CollectionError::CollectionIsFull)?;

        if let Some(timedout) = timedout {
            self.timedouts
                .insert(id, timedout)
                .map_err(|_| crate::CollectionError::CollectionIsFull)?;
        }

        Ok(())
    }

    fn drop_timedout(&mut self) {
        self.timedouts.retain(|id, timedout| {
            if Instant::now() >= *timedout {
                if let Some(ke) = self.keyexprs.remove(id) {
                    self.callbacks.remove(&(*id, ke));
                }

                false
            } else {
                true
            }
        });
    }

    fn remove(&mut self, id: u32) -> core::result::Result<(), crate::CollectionError> {
        if let Some(ke) = self.keyexprs.remove(&id) {
            self.callbacks.remove(&(id, ke));
        }
        self.timedouts.remove(&id);

        Ok(())
    }

    fn get(&mut self, id: u32) -> Option<&mut DynCallback<'a, Callback, Future, Arg>> {
        let ke = self.keyexprs.get(&id)?;
        self.callbacks.get_mut(&(id, ke))
    }

    fn intersects<'r>(
        &'r mut self,
        ke: &keyexpr,
    ) -> impl Iterator<Item = &'r mut DynCallback<'a, Callback, Future, Arg>>
    where
        DynCallback<'a, Callback, Future, Arg>: 'r,
    {
        self.callbacks
            .iter_mut()
            .filter_map(move |((_, registered_ke), callback)| {
                if registered_ke.intersects(ke) {
                    Some(callback)
                } else {
                    None
                }
            })
    }
}

pub mod storage {
    pub use dyn_utils::storage::{Raw, RawOrBox};
}

pub type FixedCapacityGetCallbacks<
    'a,
    const CAPACITY: usize,
    Callback = RawOrBox<16>,
    Future = RawOrBox<128>,
> = FixedCapacityCallbacks<'a, ResponseRef, CAPACITY, Callback, Future>;

pub type FixedCapacitySubCallbacks<
    'a,
    const CAPACITY: usize,
    Callback = RawOrBox<16>,
    Future = RawOrBox<128>,
> = FixedCapacityCallbacks<'a, SampleRef, CAPACITY, Callback, Future>;

pub struct SyncCallback<Arg, F>(F, PhantomData<Arg>);

impl<Arg, F> SyncCallback<Arg, F> {
    pub fn new(f: F) -> Self {
        Self(f, PhantomData)
    }
}

impl<Arg, F> ZCallback for SyncCallback<Arg, F>
where
    Arg: ZArg,
    F: FnMut(Arg::Of<'_>),
{
    type Arg = Arg;

    #[dyn_utils::sync]
    async fn call(&mut self, arg: <Self::Arg as ZArg>::Of<'_>) {
        (self.0)(arg)
    }
}

pub struct AsyncCallback<Arg, F>(F, PhantomData<Arg>);

impl<Arg, F> AsyncCallback<Arg, F> {
    pub fn new(f: F) -> Self {
        Self(f, PhantomData)
    }
}
impl<Arg, F> ZCallback for AsyncCallback<Arg, F>
where
    Arg: ZArg,
    F: AsyncFnMut(Arg::Of<'_>),
{
    type Arg = Arg;

    fn call(&mut self, arg: <Self::Arg as ZArg>::Of<'_>) -> impl Future<Output = ()> {
        (self.0)(arg)
    }
}

#[test]
fn test() {
    use super::Response;
    use dyn_utils::storage::RawOrBox;

    struct ResponseRef;
    impl ZArg for ResponseRef {
        type Of<'a> = &'a Response<'a>;
    }

    trait ZTestConfig {
        type GetCallbacks<'a>: ZCallbacks<'a, ResponseRef>;
    }

    struct ExampleConfig {}

    impl ZTestConfig for ExampleConfig {
        type GetCallbacks<'a> =
            FixedCapacityCallbacks<'a, ResponseRef, 8, RawOrBox<128>, RawOrBox<128>>;
    }

    struct Test {}
    let mut test = Test {};

    impl Test {
        fn borrow_mut(&mut self) {}
    }

    let mut callbacks: FixedCapacityCallbacks<ResponseRef, 8, RawOrBox<128>, RawOrBox<128>> =
        <ExampleConfig as ZTestConfig>::GetCallbacks::empty();

    callbacks
        .insert(
            9,
            keyexpr::from_str_unchecked("azd/azd"),
            None,
            DynObject::new(AsyncCallback(
                async |_: &Response<'_>| {
                    test.borrow_mut();
                },
                PhantomData,
            )),
        )
        .unwrap();
}
