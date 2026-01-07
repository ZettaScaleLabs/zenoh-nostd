pub(crate) mod recv;
pub(crate) mod send;
pub(crate) mod update;

use core::ops::{Deref, DerefMut};

use embassy_futures::select::{Either, select};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embassy_time::{Instant, Timer};
use zenoh_proto::msgs::KeepAlive;

use crate::{
    api::{SessionResources, ZConfig},
    io::transport::{TransportMineConfig, TransportOtherConfig, TransportRx, TransportTx},
};

pub struct DriverTx<'a, Config>
where
    Config: ZConfig,
{
    pub(crate) tx_buf: Config::TxBuf,
    pub(crate) tx: TransportTx<'a, Config::Platform>,
    pub(crate) sn: u32,

    pub(crate) next_keepalive: Instant,
    pub(crate) config: TransportMineConfig,
}

pub struct DriverRx<'a, Config>
where
    Config: ZConfig,
{
    pub(crate) rx_buf: Config::RxBuf,
    pub(crate) rx: TransportRx<'a, Config::Platform>,

    pub(crate) last_read: Instant,
    pub(crate) config: TransportOtherConfig,
}

pub struct Driver<'a, Config>
where
    Config: ZConfig,
{
    pub(crate) tx: Mutex<NoopRawMutex, DriverTx<'a, Config>>,
    pub(crate) rx: Mutex<NoopRawMutex, DriverRx<'a, Config>>,
}

impl<'r, Config> Driver<'r, Config>
where
    Config: ZConfig,
{
    pub(crate) fn new(tx: DriverTx<'r, Config>, rx: DriverRx<'r, Config>) -> Self {
        Self {
            tx: Mutex::new(tx),
            rx: Mutex::new(rx),
        }
    }
}

impl<Config> Driver<'_, Config>
where
    Config: ZConfig,
{
    pub(crate) async fn run(&self, resources: &SessionResources<Config>) -> crate::ZResult<()> {
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

                        tx.unframed(KeepAlive {}).await?;
                    }
                }
                Either::Second(msg) => {
                    self.update(msg?, resources).await?;
                }
            }
        }
    }
}
