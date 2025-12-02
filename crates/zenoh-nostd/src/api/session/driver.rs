pub(crate) mod recv;
pub(crate) mod send;
pub(crate) mod update;

use core::ops::{Deref, DerefMut};

use embassy_futures::select::{Either, select};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embassy_time::{Instant, Timer};
use zenoh_proto::msgs::KeepAlive;

use crate::{
    io::transport::{TransportMineConfig, TransportOtherConfig, TransportRx, TransportTx},
    platform::ZPlatform,
};

pub struct DriverTx<TxBuf, Tx> {
    pub(crate) tx_buf: TxBuf,
    pub(crate) tx: Tx,
    pub(crate) sn: u32,

    pub(crate) next_keepalive: Instant,
    pub(crate) config: TransportMineConfig,
}

pub struct DriverRx<RxBuf, Rx> {
    pub(crate) rx_buf: RxBuf,
    pub(crate) rx: Rx,

    pub(crate) last_read: Instant,
    pub(crate) config: TransportOtherConfig,
}

pub struct Driver<Tx, Rx> {
    pub(crate) tx: Mutex<NoopRawMutex, Tx>,
    pub(crate) rx: Mutex<NoopRawMutex, Rx>,
}

impl<Tx, Rx> Driver<Tx, Rx> {
    pub(crate) fn new(tx: Tx, rx: Rx) -> Self {
        Self {
            tx: Mutex::new(tx),
            rx: Mutex::new(rx),
        }
    }
}

impl<TxBuf, RxBuf, Platform>
    Driver<DriverTx<TxBuf, TransportTx<'_, Platform>>, DriverRx<RxBuf, TransportRx<'_, Platform>>>
where
    TxBuf: AsMut<[u8]>,
    RxBuf: AsMut<[u8]>,
    Platform: ZPlatform,
{
    pub async fn run(&self) -> crate::ZResult<()> {
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
                    self.update(msg?).await?;
                }
            }
        }
    }
}
