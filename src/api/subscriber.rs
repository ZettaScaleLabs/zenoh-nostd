use heapless::index_map::FnvIndexMap;

use crate::{api::sample::ZSample, protocol::core::wire_expr::WireExpr, result::ZResult};

pub enum ZCallback {
    Sync(fn(&ZSample) -> ()),
}

impl ZCallback {
    pub async fn call(&self, sample: &ZSample<'_>) {
        match self {
            ZCallback::Sync(cb) => cb(sample),
        }
    }
}

pub enum Subscriber {
    Sync,
}

pub trait ZCallbackMap {
    fn get_callback(&self, ke: WireExpr<'static>) -> Option<&ZCallback>;
    fn insert_callback(
        &mut self,
        ke: WireExpr<'static>,
        callback: ZCallback,
    ) -> ZResult<Option<ZCallback>>;

    fn iter(&self) -> heapless::index_map::Iter<'_, WireExpr<'static>, ZCallback>;
}

impl<const N: usize> ZCallbackMap for FnvIndexMap<WireExpr<'static>, ZCallback, N> {
    fn get_callback(&self, ke: WireExpr<'static>) -> Option<&ZCallback> {
        self.get(&ke)
    }

    fn insert_callback(
        &mut self,
        ke: WireExpr<'static>,
        callback: ZCallback,
    ) -> ZResult<Option<ZCallback>> {
        self.insert(ke, callback)
            .map_err(|_| crate::result::ZError::Invalid)
    }

    fn iter(&self) -> heapless::index_map::Iter<'_, WireExpr<'static>, ZCallback> {
        self.iter()
    }
}

#[macro_export]
macro_rules! zcallback {
    ($sync:expr) => {
        $crate::api::subscriber::ZCallback::Sync($sync)
    };
}
