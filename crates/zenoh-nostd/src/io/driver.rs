use core::ops::DerefMut;
use embassy_futures::select::{Either3, select3};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embassy_time::{Duration, Instant, Timer};
use zenoh_proto::{EitherError, TransportLinkError};

use crate::io::{TransportLinkRx, TransportLinkTx, ZLinkManager, ZTransportLinkRx};

pub(crate) struct Driver<'res, 'transport, LinkManager, Buff, Update>
where
    LinkManager: ZLinkManager,
{
    tx: Mutex<NoopRawMutex, TransportLinkTx<'res, 'transport, LinkManager, Buff>>,
    rx: Mutex<NoopRawMutex, TransportLinkRx<'res, 'transport, LinkManager, Buff>>,
    update: Mutex<NoopRawMutex, Update>,
}

impl<'res, 'transport, LinkManager, Buff, Update>
    Driver<'res, 'transport, LinkManager, Buff, Update>
where
    LinkManager: ZLinkManager,
{
    pub fn new(
        tx: TransportLinkTx<'res, 'transport, LinkManager, Buff>,
        rx: TransportLinkRx<'res, 'transport, LinkManager, Buff>,
        update: Update,
    ) -> Self {
        Self {
            tx: Mutex::new(tx),
            rx: Mutex::new(rx),
            update: Mutex::new(update),
        }
    }

    pub async fn run<E>(&self) -> core::result::Result<(), EitherError<TransportLinkError, E>>
    where
        Update: AsyncFnMut() -> core::result::Result<(), E>,
        Buff: AsMut<[u8]> + AsRef<[u8]>,
    {
        let mut rx_guard = self.rx.lock().await;
        let rx = rx_guard.deref_mut();

        let mut update_guard = self.update.lock().await;
        let update = update_guard.deref_mut();

        let start = Instant::now();

        loop {
            let (write_lease, read_lease) = self.sync(start, start.elapsed(), rx).await;

            match select3(write_lease, read_lease, rx.recv()).await {
                Either3::First(_) => {}
                Either3::Second(_) => {}
                Either3::Third(_) => {}
            }
        }

        Ok(())
    }

    pub async fn sync(
        &self,
        start: Instant,
        now: Duration,
        rx: &mut TransportLinkRx<'res, 'transport, LinkManager, Buff>,
    ) -> (Timer, Timer) {
        let mut tx_guard = self.tx.lock().await;
        let tx = tx_guard.deref_mut();

        rx.transport_mut().sync(Some(tx.transport()), now.into());
        tx.transport_mut().sync(Some(rx.transport()), now.into());

        let write_lease = start + tx.transport().next_timeout().try_into().unwrap();
        let read_lease = start + rx.transport().next_timeout().try_into().unwrap();

        (Timer::at(write_lease), Timer::at(read_lease))
    }
}
