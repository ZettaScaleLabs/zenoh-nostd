use core::{cell::RefCell, ops::DerefMut};
use embassy_futures::select::{Either3, select3};
use embassy_sync::{
    blocking_mutex::raw::NoopRawMutex,
    mutex::{Mutex, MutexGuard},
};
use embassy_time::{Duration, Instant, Timer};
use zenoh_proto::{EitherError, TransportLinkError, msgs::NetworkMessage};

use crate::io::{
    TransportLink, TransportLinkRx, TransportLinkTx, ZLinkManager, ZTransportLinkRx,
    ZTransportLinkTx,
};

pub(crate) struct Driver<'ext, 'transport, LinkManager, Buff>
where
    LinkManager: ZLinkManager,
{
    tx: Mutex<NoopRawMutex, TransportLinkTx<'ext, 'transport, LinkManager, Buff>>,
    rx: RefCell<TransportLinkRx<'ext, 'transport, LinkManager, Buff>>,
}

impl<'ext, 'transport, LinkManager, Buff> Driver<'ext, 'transport, LinkManager, Buff>
where
    LinkManager: ZLinkManager,
{
    pub fn new(transport: &'transport mut TransportLink<'ext, LinkManager, Buff>) -> Self {
        let (tx, rx) = transport.split();

        Self {
            tx: Mutex::new(tx),
            rx: RefCell::new(rx),
        }
    }

    pub async fn tx(
        &self,
    ) -> MutexGuard<'_, NoopRawMutex, TransportLinkTx<'ext, 'transport, LinkManager, Buff>> {
        self.tx.lock().await
    }

    pub async fn run<State, E, Update>(
        &self,
        state: &Mutex<NoopRawMutex, State>,
        mut update: Update,
    ) -> core::result::Result<(), EitherError<TransportLinkError, E>>
    where
        Buff: AsMut<[u8]> + AsRef<[u8]>,
        Update: for<'any> AsyncFnMut(
            &mut State,
            NetworkMessage<'any>,
            &'any [u8],
        ) -> core::result::Result<(), E>,
    {
        let mut rx = self.rx.borrow_mut();

        let start = Instant::now();

        loop {
            let (write_lease, read_lease) = self.sync(start, start.elapsed(), &mut rx).await;
            if rx.transport().closed() {
                return Err(EitherError::A(TransportLinkError::TransportClosed));
            }

            match select3(write_lease, read_lease, rx.recv()).await {
                Either3::First(_) => {
                    let mut tx_guard = self.tx.lock().await;
                    let tx = tx_guard.deref_mut();

                    if tx
                        .transport()
                        .should_close(start.elapsed().try_into().unwrap())
                    {
                        // TODO: send Close msg
                        break Err(EitherError::A(TransportLinkError::TransportClosed));
                    }

                    if tx
                        .transport()
                        .should_send_keepalive(start.elapsed().try_into().unwrap())
                    {
                        zenoh_proto::trace!("Sending Keepalive");
                        tx.keepalive().await?;
                    }

                    continue;
                }
                Either3::Third(res) => {
                    let mut state = state.lock().await;

                    for msg in res? {
                        update(&mut state, msg.0, msg.1)
                            .await
                            .map_err(EitherError::B)?;
                    }

                    continue;
                }
                _ => {}
            }

            if rx
                .transport()
                .should_close(start.elapsed().try_into().unwrap())
            {
                // TODO: Try send Close msg
                break Err(EitherError::A(TransportLinkError::TransportClosed));
            }
        }
    }

    pub async fn sync(
        &self,
        start: Instant,
        now: Duration,
        rx: &mut TransportLinkRx<'ext, 'transport, LinkManager, Buff>,
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
