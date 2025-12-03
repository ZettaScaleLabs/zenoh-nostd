pub(crate) mod recv;
pub(crate) mod send;
pub(crate) mod update;

use core::ops::{Deref, DerefMut};

use embassy_futures::select::{Either, select};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embassy_time::{Instant, Timer};
use zenoh_proto::msgs::KeepAlive;

use crate::{
    api::SessionResources,
    io::transport::{TransportMineConfig, TransportOtherConfig, TransportRx, TransportTx},
    platform::ZPlatform,
};

pub struct DriverTx<'a, Platform, TxBuf>
where
    Platform: ZPlatform,
{
    pub(crate) tx_buf: TxBuf,
    pub(crate) tx: TransportTx<'a, Platform>,
    pub(crate) sn: u32,

    pub(crate) next_keepalive: Instant,
    pub(crate) config: TransportMineConfig,
}

pub struct DriverRx<'a, Platform, RxBuf>
where
    Platform: ZPlatform,
{
    pub(crate) rx_buf: RxBuf,
    pub(crate) rx: TransportRx<'a, Platform>,

    pub(crate) last_read: Instant,
    pub(crate) config: TransportOtherConfig,
}

pub struct Driver<'a, Platform, TxBuf, RxBuf>
where
    Platform: ZPlatform,
{
    pub(crate) tx: Mutex<NoopRawMutex, DriverTx<'a, Platform, TxBuf>>,
    pub(crate) rx: Mutex<NoopRawMutex, DriverRx<'a, Platform, RxBuf>>,
}

impl<'a, Platform, TxBuf, RxBuf> Driver<'a, Platform, TxBuf, RxBuf>
where
    Platform: ZPlatform,
{
    pub(crate) fn new(
        tx: DriverTx<'a, Platform, TxBuf>,
        rx: DriverRx<'a, Platform, RxBuf>,
    ) -> Self {
        Self {
            tx: Mutex::new(tx),
            rx: Mutex::new(rx),
        }
    }
}

impl<'a, Platform, TxBuf, RxBuf> Driver<'a, Platform, TxBuf, RxBuf>
where
    Platform: ZPlatform,
    TxBuf: AsMut<[u8]>,
    RxBuf: AsMut<[u8]>,
{
    pub async fn run<
        const MAX_KEYEXPR_LEN: usize,
        const MAX_PARAMETERS_LEN: usize,
        const MAX_PAYLOAD_LEN: usize,
        const MAX_QUEUED: usize,
        const MAX_CALLBACKS: usize,
        const MAX_SUBSCRIBERS: usize,
    >(
        &self,
        resources: &SessionResources<
            MAX_KEYEXPR_LEN,
            MAX_PARAMETERS_LEN,
            MAX_PAYLOAD_LEN,
            MAX_QUEUED,
            MAX_CALLBACKS,
            MAX_SUBSCRIBERS,
        >,
    ) -> crate::ZResult<()> {
        let mut rx_guard = self.rx.lock().await;
        let rx = rx_guard.deref_mut();

        loop {
            let write_lease = {
                let tx_guard = self.tx.lock().await;
                let tx = tx_guard.deref();
                Timer::at(tx.next_keepalive())
            };

            match select(write_lease, rx.recv()).await {
                Either::First(_) => {
                    let mut tx_guard = self.tx.lock().await;
                    let tx = tx_guard.deref_mut();
                    if Instant::now() >= tx.next_keepalive() {
                        zenoh_proto::trace!("Sending KeepAlive");

                        tx.unframe(KeepAlive {}).await?;
                    }
                }
                Either::Second(msg) => {
                    self.update(msg?, resources).await?;
                }
            }
        }
    }
}
