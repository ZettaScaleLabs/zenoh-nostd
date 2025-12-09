pub(crate) mod recv;
pub(crate) mod send;
pub(crate) mod update;

use core::ops::{Deref, DerefMut};

use embassy_futures::select::{Either, select};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embassy_time::{Instant, Timer};
use zenoh_proto::msgs::KeepAlive;

use crate::{
    api::{SessionResources, ZDriverConfig, ZSessionConfig},
    io::transport::{TransportMineConfig, TransportOtherConfig, TransportRx, TransportTx},
};

pub struct DriverTx<'a, DriverConfig>
where
    DriverConfig: ZDriverConfig,
{
    pub(crate) tx_buf: DriverConfig::TxBuf,
    pub(crate) tx: TransportTx<'a, DriverConfig::Platform>,
    pub(crate) sn: u32,

    pub(crate) next_keepalive: Instant,
    pub(crate) config: TransportMineConfig,
}

pub struct DriverRx<'a, DriverConfig>
where
    DriverConfig: ZDriverConfig,
{
    pub(crate) rx_buf: DriverConfig::RxBuf,
    pub(crate) rx: TransportRx<'a, DriverConfig::Platform>,

    pub(crate) last_read: Instant,
    pub(crate) config: TransportOtherConfig,
}

pub struct Driver<'a, DriverConfig>
where
    DriverConfig: ZDriverConfig,
{
    pub(crate) tx: Mutex<NoopRawMutex, DriverTx<'a, DriverConfig>>,
    pub(crate) rx: Mutex<NoopRawMutex, DriverRx<'a, DriverConfig>>,
}

impl<'r, DriverConfig> Driver<'r, DriverConfig>
where
    DriverConfig: ZDriverConfig,
{
    pub(crate) fn new(tx: DriverTx<'r, DriverConfig>, rx: DriverRx<'r, DriverConfig>) -> Self {
        Self {
            tx: Mutex::new(tx),
            rx: Mutex::new(rx),
        }
    }
}

impl<Config> Driver<'_, Config>
where
    Config: ZDriverConfig + ZSessionConfig,
{
    pub async fn run(&self, resources: &SessionResources<Config>) -> crate::ZResult<()> {
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
