use crate::api::subscriber::ZCallbackMap;

pub mod sample;

pub mod driver;
pub mod publisher;
pub mod session;
pub mod subscriber;

pub struct ZConfig<T> {
    pub platform: T,

    pub tx_zbuf: &'static mut [u8],
    pub rx_zbuf: &'static mut [u8],

    pub callbacks: &'static mut dyn ZCallbackMap,
}
