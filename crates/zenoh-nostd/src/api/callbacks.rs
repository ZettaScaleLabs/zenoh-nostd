use core::{
    mem::MaybeUninit,
    pin::Pin,
    task::{Context, Poll},
};

use heapless::FnvIndexMap;
use zenoh_proto::keyexpr;

struct CallbackFuture<const ASYNC_CALLBACK_MEMORY: usize, Output> {
    fut: MaybeUninit<[u8; ASYNC_CALLBACK_MEMORY]>,

    drop_future: unsafe fn(*mut ()),
    poll_future: unsafe fn(*mut (), &mut Context<'_>) -> Poll<Output>,

    _pin: core::marker::PhantomPinned,
}

impl<const ASYNC_CALLBACK_MEMORY: usize, Output> Drop
    for CallbackFuture<ASYNC_CALLBACK_MEMORY, Output>
{
    fn drop(&mut self) {
        unsafe {
            (self.drop_future)(self.fut.as_ptr() as *mut ());
        }
    }
}

impl<const ASYNC_CALLBACK_MEMORY: usize, Output> Future
    for CallbackFuture<ASYNC_CALLBACK_MEMORY, Output>
{
    type Output = Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe { (self.poll_future)(self.fut.as_ptr() as *mut (), cx) }
    }
}

unsafe fn trampoline_call<F, Fut, A, B>(ctx: *mut (), data: *mut (), arg: A)
where
    F: Fn(A) -> Fut,
    Fut: Future<Output = B>,
{
    let f: &F = unsafe { &*(ctx as *const F) };
    let result = f(arg);

    unsafe {
        core::ptr::write(data as *mut _, result);
    }
}

unsafe fn trampoline_drop<Fut, B>(fut: *mut ())
where
    Fut: Future<Output = B>,
{
    let fut: &mut Fut = unsafe { &mut *(fut as *mut Fut) };
    unsafe {
        core::ptr::drop_in_place(fut);
    }
}

unsafe fn trampoline_poll<Fut, B>(fut: *mut (), cx: &mut Context<'_>) -> Poll<B>
where
    Fut: Future<Output = B>,
{
    let fut: &mut Fut = unsafe { &mut *(fut as *mut Fut) };
    let mut fut = unsafe { Pin::new_unchecked(fut) };
    fut.as_mut().poll(cx)
}

struct AsyncCallback<A, B, const ASYNC_CALLBACK_MEMORY: usize> {
    ctx: *mut (),

    call: unsafe fn(*mut (), *mut (), A),
    drop_future: unsafe fn(*mut ()),
    poll_future: unsafe fn(*mut (), &mut Context<'_>) -> Poll<B>,
}

impl<A, B, const ASYNC_CALLBACK_MEMORY: usize> AsyncCallback<A, B, ASYNC_CALLBACK_MEMORY> {
    pub fn new<F, Fut>(f: &'static F) -> Self
    where
        F: Fn(A) -> Fut + 'static,
        Fut: Future<Output = B>,
    {
        const {
            assert!(
                core::mem::size_of::<Fut>() <= ASYNC_CALLBACK_MEMORY,
                "Stored future is too large for the provided storage size"
            );
        };

        AsyncCallback {
            ctx: f as *const F as *mut (),
            call: trampoline_call::<F, Fut, A, B>,
            drop_future: trampoline_drop::<Fut, B>,
            poll_future: trampoline_poll::<Fut, B>,
        }
    }

    pub fn execute(&self, arg: A) -> CallbackFuture<ASYNC_CALLBACK_MEMORY, B> {
        let mut fut: MaybeUninit<[u8; ASYNC_CALLBACK_MEMORY]> = MaybeUninit::uninit();

        unsafe {
            (self.call)(self.ctx, fut.as_mut_ptr() as *mut (), arg);
        }

        CallbackFuture {
            fut,
            drop_future: self.drop_future,
            poll_future: self.poll_future,
            _pin: core::marker::PhantomPinned,
        }
    }
}

pub struct SyncCallback<A, B> {
    call: fn(A) -> B,
}

impl<A, B> SyncCallback<A, B> {
    pub fn new(f: fn(A) -> B) -> Self {
        SyncCallback { call: f }
    }

    pub fn execute(&self, arg: A) -> B {
        (self.call)(arg)
    }
}

pub trait ZCallback<A, B> {
    fn execute(&self, arg: A) -> impl Future<Output = B>;
}

impl<A, B, const ASYNC_CALLBACK_MEMORY: usize> ZCallback<A, B>
    for AsyncCallback<A, B, ASYNC_CALLBACK_MEMORY>
{
    fn execute(&self, arg: A) -> impl Future<Output = B> {
        self.execute(arg)
    }
}

impl<A, B> ZCallback<A, B> for SyncCallback<A, B> {
    fn execute(&self, arg: A) -> impl Future<Output = B> {
        core::future::ready(self.execute(arg))
    }
}

enum CallbackInner<A, B, const ASYNC_CALLBACK_MEMORY: usize> {
    Sync(SyncCallback<A, B>),
    Async(AsyncCallback<A, B, ASYNC_CALLBACK_MEMORY>),
}

pub struct Callback<A, B, const ASYNC_CALLBACK_MEMORY: usize>(
    CallbackInner<A, B, ASYNC_CALLBACK_MEMORY>,
);

impl<A, B, const ASYNC_CALLBACK_MEMORY: usize> ZCallback<A, B>
    for Callback<A, B, ASYNC_CALLBACK_MEMORY>
{
    async fn execute(&self, arg: A) -> B {
        match &self.0 {
            CallbackInner::Sync(sync_cb) => ZCallback::execute(sync_cb, arg).await,
            CallbackInner::Async(async_cb) => ZCallback::execute(async_cb, arg).await,
        }
    }
}

impl<A, B, const ASYNC_CALLBACK_MEMORY: usize> Callback<A, B, ASYNC_CALLBACK_MEMORY> {
    pub fn from_sync(f: fn(A) -> B) -> Self {
        Callback(CallbackInner::Sync(SyncCallback::new(f)))
    }

    pub fn from_async<F, Fut>(f: &'static F) -> Self
    where
        F: Fn(A) -> Fut + 'static,
        Fut: Future<Output = B>,
    {
        Callback(CallbackInner::Async(AsyncCallback::new::<F, Fut>(f)))
    }
}

pub trait ZCallbacks<A, B> {
    type Callback: ZCallback<A, B>;

    fn empty() -> Self;
    fn insert(
        &mut self,
        id: u32,
        ke: &'static keyexpr,
        callback: impl Into<Self::Callback>,
    ) -> core::result::Result<(), crate::CollectionError>;
    fn remove(&mut self, id: u32) -> core::result::Result<(), crate::CollectionError>;
    fn intersects<'r>(&'r self, ke: &keyexpr) -> impl Iterator<Item = &'r Self::Callback>
    where
        Self::Callback: 'r,
        A: 'r,
        B: 'r;
}

pub struct HeaplessCallbacks<A, B, const ASYNC_CALLBACK_MEMORY: usize, const CAPACITY: usize> {
    keyexprs: FnvIndexMap<u32, &'static keyexpr, CAPACITY>,
    callbacks: FnvIndexMap<u32, Callback<A, B, ASYNC_CALLBACK_MEMORY>, CAPACITY>,
}

impl<A, B, const ASYNC_CALLBACK_MEMORY: usize, const CAPACITY: usize> ZCallbacks<A, B>
    for HeaplessCallbacks<A, B, ASYNC_CALLBACK_MEMORY, CAPACITY>
{
    type Callback = Callback<A, B, ASYNC_CALLBACK_MEMORY>;

    fn empty() -> Self {
        Self {
            keyexprs: FnvIndexMap::new(),
            callbacks: FnvIndexMap::new(),
        }
    }

    fn insert(
        &mut self,
        id: u32,
        ke: &'static keyexpr,
        callback: impl Into<Self::Callback>,
    ) -> core::result::Result<(), crate::CollectionError> {
        if self.keyexprs.contains_key(&id) {
            return Err(crate::CollectionError::KeyAlreadyExists);
        }

        if self.callbacks.contains_key(&id) {
            return Err(crate::CollectionError::KeyAlreadyExists);
        }

        self.keyexprs
            .insert(id, ke)
            .map_err(|_| crate::CollectionError::CollectionIsFull)?;

        self.callbacks
            .insert(id, callback.into())
            .map_err(|_| crate::CollectionError::CollectionIsFull)?;

        Ok(())
    }

    fn remove(&mut self, id: u32) -> core::result::Result<(), crate::CollectionError> {
        self.keyexprs.remove(&id);
        self.callbacks.remove(&id);
        Ok(())
    }

    fn intersects<'r>(&'r self, ke: &keyexpr) -> impl Iterator<Item = &'r Self::Callback>
    where
        Self::Callback: 'r,
    {
        self.keyexprs.iter().filter_map(|(id, registered_ke)| {
            if registered_ke.intersects(ke) {
                self.callbacks.get(id)
            } else {
                None
            }
        })
    }
}
