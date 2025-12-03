use embassy_sync::{blocking_mutex::raw::NoopRawMutex, channel::Channel};

pub(crate) enum Callback<T, G, const N: usize> {
    Sync(T),
    Async(Channel<NoopRawMutex, G, N>),
}

impl<T, G, const N: usize> Callback<T, G, N> {
    pub(crate) fn new_sync(cb: T) -> Self {
        Self::Sync(cb)
    }

    pub(crate) fn new_async(sender: Channel<NoopRawMutex, G, N>) -> Self {
        Self::Async(sender)
    }
}

pub(crate) trait ZCallback<Arg> {
    fn call(&self, msg: Arg) -> impl ::core::future::Future<Output = crate::ZResult<()>>;
}

impl<Arg, G, const N: usize> ZCallback<Arg> for Callback<fn(&Arg), G, N>
where
    G: TryFrom<Arg, Error = crate::ZError>,
{
    async fn call(&self, msg: Arg) -> crate::ZResult<()> {
        match self {
            Callback::Sync(cb) => {
                cb(&msg);
                Ok(())
            }
            Callback::Async(sender) => {
                let owned_msg = G::try_from(msg)?;
                sender.send(owned_msg).await;
                Ok(())
            }
        }
    }
}
