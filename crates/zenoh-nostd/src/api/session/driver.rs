pub(crate) mod recv;
pub(crate) mod send;
pub(crate) mod update;

use core::ops::{Deref, DerefMut};

use embassy_futures::select::{Either, select};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embassy_time::{Instant, Timer};
use zenoh_proto::{Framed, Unframed, msgs::KeepAlive};

use crate::io::transport::{
    TransportMineConfig, TransportOtherConfig, ZTransportRecv, ZTransportSend,
};

pub struct DriverTx<TxBuf: AsMut<[u8]>, Tx: ZTransportSend> {
    pub(crate) tx_buf: TxBuf,
    pub(crate) tx: Tx,
    pub(crate) sn: u32,

    pub(crate) next_keepalive: Instant,
    pub(crate) config: TransportMineConfig,
}

pub trait ZDriverTx {
    fn frame(&mut self, x: impl Framed)
    -> impl ::core::future::Future<Output = crate::ZResult<()>>;

    fn unframe(
        &mut self,
        x: impl Unframed,
    ) -> impl ::core::future::Future<Output = crate::ZResult<()>>;

    fn next_keepalive(&self) -> Instant;
}

pub struct DriverRx<RxBuf: AsMut<[u8]>, Rx: ZTransportRecv> {
    pub(crate) rx_buf: RxBuf,
    pub(crate) rx: Rx,

    pub(crate) last_read: Instant,
    pub(crate) config: TransportOtherConfig,
}

pub trait ZDriverRx {
    fn recv(&mut self) -> impl ::core::future::Future<Output = crate::ZResult<&[u8]>>;
}

pub struct Driver<Tx: ZDriverTx, Rx: ZDriverRx> {
    pub(crate) tx: Mutex<NoopRawMutex, Tx>,
    pub(crate) rx: Mutex<NoopRawMutex, Rx>,
}

pub trait ZDriver {
    fn send(&self, x: impl Framed) -> impl ::core::future::Future<Output = crate::ZResult<()>>;
    fn run(&self) -> impl ::core::future::Future<Output = crate::ZResult<()>>;
}

impl<Tx: ZDriverTx, Rx: ZDriverRx> Driver<Tx, Rx> {
    pub(crate) fn new(tx: Tx, rx: Rx) -> Self {
        Self {
            tx: Mutex::new(tx),
            rx: Mutex::new(rx),
        }
    }
}

impl<Tx: ZDriverTx, Rx: ZDriverRx> ZDriver for Driver<Tx, Rx> {
    async fn send(&self, x: impl Framed) -> crate::ZResult<()> {
        let mut tx_guard = self.tx.lock().await;
        let tx = tx_guard.deref_mut();

        tx.frame(x).await
    }

    async fn run(&self) -> crate::ZResult<()> {
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
