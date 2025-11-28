use core::ops::{Deref, DerefMut};

use embassy_futures::select::select;
use embassy_time::{Instant, Timer};
use zenoh_proto::{ZBatchUnframed, ZError, ZResult, msgs::KeepAlive};

use crate::{api::driver::SessionDriver, platform::Platform};

impl<T: Platform> SessionDriver<T> {
    pub async fn run(&'static self) -> ZResult<()> {
        let mut rx_guard = self.rx.lock().await;
        let rx = rx_guard.deref_mut();

        let keep_alive_timeout =
            self.config.mine_config.mine_lease / (self.config.mine_config.keep_alive as u32);
        let other_lease = self.config.other_config.other_lease;

        let mut last_read_time = embassy_time::Instant::now();

        loop {
            let read_lease = Timer::at(last_read_time + other_lease.try_into().unwrap());
            let write_lease = {
                let tx_guard = self.tx.lock().await;
                let tx = tx_guard.deref();
                Timer::at(tx.next_keepalive)
            };

            match select(select(read_lease, rx.rx.recv(rx.rx_zbuf)), write_lease).await {
                embassy_futures::select::Either::First(read) => {
                    last_read_time = embassy_time::Instant::now();
                    match read {
                        embassy_futures::select::Either::First(_) => {
                            zenoh_proto::warn!("Connection closed by peer");
                            break Err(ZError::ConnectionClosed);
                        }
                        embassy_futures::select::Either::Second(tmsg) => match tmsg {
                            Ok(tmsg) => {
                                self.internal_update(tmsg).await?;
                            }
                            Err(_) => {
                                zenoh_proto::warn!("Did not read from connection");
                                break Err(ZError::ConnectionClosed);
                            }
                        },
                    }
                }
                embassy_futures::select::Either::Second(_) => {
                    let mut tx_guard = self.tx.lock().await;
                    let tx = tx_guard.deref_mut();
                    if Instant::now() >= tx.next_keepalive {
                        zenoh_proto::trace!("Sending KeepAlive");

                        tx.tx
                            .send(tx.tx_zbuf, &mut 0, |batch| batch.unframe(&KeepAlive {}))
                            .await?;

                        tx.next_keepalive = Instant::now() + keep_alive_timeout.try_into().unwrap();
                    }
                }
            }
        }
    }
}
