use static_cell::StaticCell;

enum CallbackInner<F1, F2, F3> {
    F1(F1),
    F2(F2),
    F3(F3),
}

struct Callback<A, B, F1, F2, F3>
where
    F1: AsyncFnMut(A) -> B,
    F2: AsyncFnMut(A) -> B,
    F3: AsyncFnMut(A) -> B,
{
    inner: CallbackInner<F1, F2, F3>,
    _phantom: ::core::marker::PhantomData<(A, B)>,
}

impl<A, B, F1, F2, F3> Callback<A, B, F1, F2, F3>
where
    F1: AsyncFnMut(A) -> B,
    F2: AsyncFnMut(A) -> B,
    F3: AsyncFnMut(A) -> B,
{
    fn new_f1(f: F1) -> Self {
        Self {
            inner: CallbackInner::F1(f),
            _phantom: ::core::marker::PhantomData,
        }
    }

    fn new_f2(f: F2) -> Self {
        Self {
            inner: CallbackInner::F2(f),
            _phantom: ::core::marker::PhantomData,
        }
    }

    fn new_f3(f: F3) -> Self {
        Self {
            inner: CallbackInner::F3(f),
            _phantom: ::core::marker::PhantomData,
        }
    }

    async fn call(&mut self, arg: A) -> B {
        match &mut self.inner {
            CallbackInner::F1(f) => f(arg).await,
            CallbackInner::F2(f) => f(arg).await,
            CallbackInner::F3(f) => f(arg).await,
        }
    }
}

trait ZStorage<A, B, C> {
    fn store(&mut self, callback: C);
    fn iter(&self) -> core::slice::Iter<'_, C>;
    fn iter_mut(&mut self) -> core::slice::IterMut<'_, C>;
    async fn call(&mut self, i: usize, arg: A) -> B;
}

impl<A, B, F1, F2, F3> ZStorage<A, B, Callback<A, B, F1, F2, F3>>
    for Vec<Callback<A, B, F1, F2, F3>>
where
    F1: AsyncFnMut(A) -> B,
    F2: AsyncFnMut(A) -> B,
    F3: AsyncFnMut(A) -> B,
{
    fn store(&mut self, callback: Callback<A, B, F1, F2, F3>) {
        self.push(callback);
    }

    fn iter(&self) -> core::slice::Iter<'_, Callback<A, B, F1, F2, F3>> {
        self.as_slice().iter()
    }

    fn iter_mut(&mut self) -> core::slice::IterMut<'_, Callback<A, B, F1, F2, F3>> {
        self.as_mut_slice().iter_mut()
    }

    async fn call(&mut self, i: usize, arg: A) -> B {
        self[i].call(arg).await
    }
}

struct U32CallbackStorage<S, C>
where
    S: ZStorage<u32, u32, C>,
{
    storage: S,
    _phantom: ::core::marker::PhantomData<C>,
}

impl<S, C> U32CallbackStorage<S, C>
where
    S: ZStorage<u32, u32, C>,
{
    fn new(storage: S) -> Self {
        Self {
            storage,
            _phantom: ::core::marker::PhantomData,
        }
    }

    fn add_callback(&mut self, callback: C) {
        self.storage.store(callback);
    }

    fn iter(&self) -> core::slice::Iter<'_, C> {
        self.storage.iter()
    }

    fn iter_mut(&mut self) -> core::slice::IterMut<'_, C> {
        self.storage.iter_mut()
    }

    async fn call(&mut self, i: usize, arg: u32) -> u32 {
        self.storage.call(i, arg).await
    }
}

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let mut value = 0u32;
    let data = 7u8;
    let a = async |x: u32| x + (data as u32) * 2;
    let b = async |x: u32| x * (data as u32);
    let c = async |x: u32| {
        value = 17;
        x - (data as u32)
    };

    let mut callbacks = U32CallbackStorage::new(Vec::new());
    callbacks.add_callback(Callback::new_f1(a));
    callbacks.add_callback(Callback::new_f2(b));
    callbacks.add_callback(Callback::new_f3(c));

    for i in 0..3 {
        let result = callbacks.call(i, 10).await;
        println!("Callback {}: Result = {}", i + 1, result);
    }

    for c in callbacks.iter_mut() {
        let result = c.call(10).await;
        println!("Iterated Callback: Result = {}", result);
    }

    println!("Final value: {}", value);

    #[derive(Debug)]
    struct TestStruct {
        a: u32,
    }
    let mut test_value = TestStruct { a: 42 };
}
