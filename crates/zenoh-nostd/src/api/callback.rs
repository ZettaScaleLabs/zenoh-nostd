use core::{
    mem::MaybeUninit,
    pin::Pin,
    task::{Context, Poll},
};

use heapless::FnvIndexMap;
use zenoh_proto::keyexpr;

pub struct CallbackFuture<const N: usize, Output> {
    fut: MaybeUninit<[u8; N]>,

    drop_future: unsafe fn(*mut ()),
    poll_future: unsafe fn(*mut (), &mut Context<'_>) -> Poll<Output>,

    _pin: core::marker::PhantomPinned,
}

impl<const N: usize, Output> Drop for CallbackFuture<N, Output> {
    fn drop(&mut self) {
        unsafe {
            (self.drop_future)(self.fut.as_ptr() as *mut ());
        }
    }
}

impl<const N: usize, Output> Future for CallbackFuture<N, Output> {
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

unsafe fn trampoline_poll<F, Fut, A, B>(fut: *mut (), cx: &mut Context<'_>) -> Poll<B>
where
    F: Fn(A) -> Fut,
    Fut: Future<Output = B>,
{
    let fut: &mut Fut = unsafe { &mut *(fut as *mut Fut) };
    let mut fut = unsafe { Pin::new_unchecked(fut) };
    fut.as_mut().poll(cx)
}

pub struct CallbackStruct<A, B, const N: usize> {
    ctx: *mut (),

    call: unsafe fn(*mut (), *mut (), A),
    drop_future: unsafe fn(*mut ()),
    poll_future: unsafe fn(*mut (), &mut Context<'_>) -> Poll<B>,
}

impl<A, B, const N: usize> CallbackStruct<A, B, N> {
    pub fn new<F, Fut>(f: &'static F) -> Self
    where
        F: Fn(A) -> Fut + 'static,
        Fut: Future<Output = B>,
    {
        const {
            assert!(
                core::mem::size_of::<Fut>() <= N,
                "Stored future is too large for the provided storage size"
            );
        };

        CallbackStruct {
            ctx: f as *const F as *mut (),
            call: trampoline_call::<F, Fut, A, B>,
            drop_future: trampoline_drop::<Fut, B>,
            poll_future: trampoline_poll::<F, Fut, A, B>,
        }
    }

    pub fn execute(&self, arg: A) -> CallbackFuture<N, B> {
        let mut fut: MaybeUninit<[u8; N]> = MaybeUninit::uninit();

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

impl<A, B, const N: usize, F, Fut> From<&'static F> for CallbackStruct<A, B, N>
where
    F: Fn(A) -> Fut + 'static,
    Fut: Future<Output = B>,
{
    fn from(f: &'static F) -> Self {
        Self::new(f)
    }
}

pub trait ZCallback<A, B> {
    fn execute(&self, arg: A) -> impl Future<Output = B>;
}

impl<A, B, const CALLBACK_MEMORY: usize> ZCallback<A, B> for CallbackStruct<A, B, CALLBACK_MEMORY> {
    fn execute(&self, arg: A) -> impl Future<Output = B> {
        self.execute(arg)
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
    ) -> crate::ZResult<()>;
    fn remove(&mut self, id: u32) -> crate::ZResult<()>;
    fn intersects<'a>(&'a self, ke: &keyexpr) -> impl Iterator<Item = &'a Self::Callback>
    where
        Self::Callback: 'a,
        A: 'a,
        B: 'a;
}

pub struct HeaplessCallbacks<A, B, const CALLBACK_MEMORY: usize, const CAPACITY: usize> {
    keyexprs: FnvIndexMap<u32, &'static keyexpr, CAPACITY>,
    callbacks: FnvIndexMap<u32, CallbackStruct<A, B, CALLBACK_MEMORY>, CAPACITY>,
}

impl<A, B, const CALLBACK_MEMORY: usize, const CAPACITY: usize> ZCallbacks<A, B>
    for HeaplessCallbacks<A, B, CALLBACK_MEMORY, CAPACITY>
{
    type Callback = CallbackStruct<A, B, CALLBACK_MEMORY>;

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
    ) -> crate::ZResult<()> {
        self.keyexprs
            .insert(id, ke)
            .map_err(|_| crate::CollectionError::CollectionIsFull)?;
        self.callbacks
            .insert(id, callback.into())
            .map_err(|_| crate::CollectionError::CollectionIsFull)?;
        Ok(())
    }

    fn remove(&mut self, id: u32) -> crate::ZResult<()> {
        self.keyexprs.remove(&id);
        self.callbacks.remove(&id);
        Ok(())
    }

    fn intersects<'a>(&'a self, ke: &keyexpr) -> impl Iterator<Item = &'a Self::Callback>
    where
        Self::Callback: 'a,
    {
        self.keyexprs.iter().filter_map(move |(id, registered_ke)| {
            if registered_ke.intersects(ke) {
                self.callbacks.get(id)
            } else {
                None
            }
        })
    }
}
