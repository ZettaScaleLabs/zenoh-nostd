use crate::platform::ZPlatform;

pub trait ZConfig: 'static {
    type Platform: ZPlatform;

    // type SubscriberCallbacks: ZCallbacks<SampleRef, ()>;
    // type GetCallbacks: ZCallbacks<ResponseRef, ()>;
    // type QueryableCallbacks: ZCallbacks<QueryRef<Self>, ()>
    // where
    //     Self: Sized + 'static;

    type TxBuf: AsMut<[u8]>;
    type RxBuf: AsMut<[u8]>;

    fn platform(&self) -> &Self::Platform;

    fn txrx(&mut self) -> (&mut Self::TxBuf, &mut Self::RxBuf);

    fn into_parts(self) -> (Self::Platform, Self::TxBuf, Self::RxBuf);
}
