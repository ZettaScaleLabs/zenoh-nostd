use core::{mem::MaybeUninit, pin::Pin, ptr::NonNull};

use elain::{Align, Alignment};

pub trait ZCallback<Arg, Ret> {
    fn call(&self, arg: Arg) -> impl Future<Output = Ret>;
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
        crate::trace!(
            "Creating DynStorage for type of size {} and alignment {}",
            size_of::<T>(),
            align_of::<T>()
        );

        const {
            assert!(size_of::<T>() <= SIZE);
            assert!(align_of::<T>() <= ALIGN);
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

enum ReturnOrFuture<Ret, Fut> {
    Return(Ret),
    Future(Fut),
}

pub struct AsyncCallback<
    Arg,
    Ret,
    const CALLBACK_SIZE: usize = { size_of::<usize>() },
    const CALLBACK_ALIGN: usize = { size_of::<usize>() },
    const FUTURE_SIZE: usize = { 4 * size_of::<usize>() },
    const FUTURE_ALIGN: usize = { size_of::<usize>() },
> where
    Align<CALLBACK_ALIGN>: Alignment,
    Align<FUTURE_ALIGN>: Alignment,
{
    context: DynStorage<CALLBACK_SIZE, CALLBACK_ALIGN>,
    #[expect(clippy::type_complexity)]
    callback: unsafe fn(
        NonNull<()>,
        Arg,
        &mut Option<DynStorage<FUTURE_SIZE, FUTURE_ALIGN>>,
    ) -> ReturnOrFuture<Ret, Pin<&mut dyn Future<Output = Ret>>>,
}

impl<
    Arg,
    Ret,
    const CALLBACK_SIZE: usize,
    const CALLBACK_ALIGN: usize,
    const FUTURE_SIZE: usize,
    const FUTURE_ALIGN: usize,
> AsyncCallback<Arg, Ret, CALLBACK_SIZE, CALLBACK_ALIGN, FUTURE_SIZE, FUTURE_ALIGN>
where
    Align<CALLBACK_ALIGN>: Alignment,
    Align<FUTURE_ALIGN>: Alignment,
{
    pub(crate) fn new_async<F: AsyncFn(Arg) -> Ret>(f: F) -> Self {
        Self {
            context: DynStorage::new(f),
            callback:
                |ctx: NonNull<()>,
                 arg: Arg,
                 fut: &mut Option<DynStorage<FUTURE_SIZE, FUTURE_ALIGN>>| unsafe {
                    let fut = DynStorage::insert(fut, ctx.cast::<F>().as_ref()(arg));
                    core::mem::transmute(ReturnOrFuture::Future::<Ret, _>(Pin::new_unchecked(
                        fut as &mut dyn Future<Output = Ret>,
                    )))
                },
        }
    }

    pub(crate) fn new_sync<F: Fn(Arg) -> Ret>(f: F) -> Self {
        Self {
            context: DynStorage::new(f),
            callback:
                |ctx: NonNull<()>,
                 arg: Arg,
                 _: &mut Option<DynStorage<FUTURE_SIZE, FUTURE_ALIGN>>| unsafe {
                    ReturnOrFuture::Return(ctx.cast::<F>().as_ref()(arg))
                },
        }
    }
}

impl<
    Arg,
    Ret,
    const CALLBACK_SIZE: usize,
    const CALLBACK_ALIGN: usize,
    const FUTURE_SIZE: usize,
    const FUTURE_ALIGN: usize,
> ZCallback<Arg, Ret>
    for AsyncCallback<Arg, Ret, CALLBACK_SIZE, CALLBACK_ALIGN, FUTURE_SIZE, FUTURE_ALIGN>
where
    Align<CALLBACK_ALIGN>: Alignment,
    Align<FUTURE_ALIGN>: Alignment,
{
    async fn call(&self, arg: Arg) -> Ret {
        let mut future = None;
        match unsafe { (self.callback)(self.context.ptr(), arg, &mut future) } {
            ReturnOrFuture::Return(ret) => ret,
            ReturnOrFuture::Future(fut) => fut.await,
        }
    }
}
