use core::{mem::MaybeUninit, pin::Pin, ptr::NonNull};

use elain::{Align, Alignment};
use higher_kinded_types::ForLt;

pub trait ZCallback<Arg: ForLt, Ret> {
    fn call(&mut self, arg: Arg::Of<'_>) -> impl Future<Output = Ret>;
}

enum ReturnOrFuture<Ret, Fut> {
    Return(Ret),
    Future(Fut),
}

pub struct Callback<
    Arg: ForLt,
    Ret,
    const CALLBACK_SIZE: usize = { size_of::<usize>() },
    const FUTURE_SIZE: usize = { 4 * size_of::<usize>() },
    const CALLBACK_ALIGN: usize = { size_of::<usize>() },
    const FUTURE_ALIGN: usize = { size_of::<usize>() },
> where
    Align<CALLBACK_ALIGN>: Alignment,
    Align<FUTURE_ALIGN>: Alignment,
{
    context: DynStorage<CALLBACK_SIZE, CALLBACK_ALIGN>,
    #[expect(clippy::type_complexity)]
    callback: for<'a, 'b> unsafe fn(
        NonNull<()>,
        Arg::Of<'a>,
        &'b mut Option<DynStorage<FUTURE_SIZE, FUTURE_ALIGN>>,
    )
        -> ReturnOrFuture<Ret, Pin<&'b mut dyn Future<Output = Ret>>>,
}

impl<
    Arg,
    Ret,
    const CALLBACK_SIZE: usize,
    const FUTURE_SIZE: usize,
    const CALLBACK_ALIGN: usize,
    const FUTURE_ALIGN: usize,
> Callback<Arg, Ret, CALLBACK_SIZE, FUTURE_SIZE, CALLBACK_ALIGN, FUTURE_ALIGN>
where
    Arg: ForLt,
    Align<CALLBACK_ALIGN>: Alignment,
    Align<FUTURE_ALIGN>: Alignment,
{
    pub fn new_async<F: AsyncFnMut(Arg::Of<'_>) -> Ret>(f: F) -> Self {
        Self {
            context: DynStorage::new(f),
            callback:
                |ctx: NonNull<()>,
                 arg: Arg::Of<'_>,
                 fut: &mut Option<DynStorage<FUTURE_SIZE, FUTURE_ALIGN>>| unsafe {
                    let fut = DynStorage::insert(fut, ctx.cast::<F>().as_mut()(arg));
                    core::mem::transmute(ReturnOrFuture::Future::<Ret, _>(Pin::new_unchecked(
                        fut as &mut dyn Future<Output = Ret>,
                    )))
                },
        }
    }

    pub fn new_sync<F: FnMut(Arg::Of<'_>) -> Ret>(f: F) -> Self {
        Self {
            context: DynStorage::new(f),
            callback:
                |ctx: NonNull<()>,
                 arg: Arg::Of<'_>,
                 _: &mut Option<DynStorage<FUTURE_SIZE, FUTURE_ALIGN>>| unsafe {
                    ReturnOrFuture::Return(ctx.cast::<F>().as_mut()(arg))
                },
        }
    }
}

impl<
    Arg,
    Ret,
    const CALLBACK_SIZE: usize,
    const FUTURE_SIZE: usize,
    const CALLBACK_ALIGN: usize,
    const FUTURE_ALIGN: usize,
> ZCallback<Arg, Ret>
    for Callback<Arg, Ret, CALLBACK_SIZE, FUTURE_SIZE, CALLBACK_ALIGN, FUTURE_ALIGN>
where
    Arg: ForLt,
    Align<CALLBACK_ALIGN>: Alignment,
    Align<FUTURE_ALIGN>: Alignment,
{
    async fn call(&mut self, arg: Arg::Of<'_>) -> Ret {
        let mut future = None;
        match unsafe { (self.callback)(self.context.ptr(), arg, &mut future) } {
            ReturnOrFuture::Return(ret) => ret,
            ReturnOrFuture::Future(fut) => fut.await,
        }
    }
}

#[repr(C)]
struct DynStorage<const SIZE: usize, const ALIGN: usize>
where
    Align<ALIGN>: Alignment,
{
    data: MaybeUninit<[u8; SIZE]>,
    drop: unsafe fn(NonNull<()>),
    _align: Align<ALIGN>,
}

impl<const SIZE: usize, const ALIGN: usize> Drop for DynStorage<SIZE, ALIGN>
where
    Align<ALIGN>: Alignment,
{
    fn drop(&mut self) {
        unsafe { (self.drop)(NonNull::from(&mut self.data).cast()) }
    }
}

impl<const SIZE: usize, const ALIGN: usize> DynStorage<SIZE, ALIGN>
where
    Align<ALIGN>: Alignment,
{
    fn new<T>(data: T) -> Self {
        const {
            assert!(size_of::<T>() <= SIZE && align_of::<T>() <= ALIGN);
        };
        let mut this = Self {
            data: MaybeUninit::uninit(),
            drop: |data: NonNull<()>| unsafe { data.cast::<T>().drop_in_place() },
            _align: Align::default(),
        };
        unsafe { this.data.as_mut_ptr().cast::<T>().write(data) };
        this
    }

    fn insert<T>(option: &mut Option<Self>, data: T) -> &mut T {
        let ptr = NonNull::from(&mut option.insert(Self::new(data)).data);
        unsafe { ptr.cast().as_mut() }
    }

    fn ptr(&self) -> NonNull<()> {
        NonNull::from(&self.data).cast()
    }
}
