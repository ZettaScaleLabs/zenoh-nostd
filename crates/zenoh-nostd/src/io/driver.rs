use core::{cell::RefCell, ops::DerefMut};
use embassy_futures::select::{Either3, select3};
use embassy_sync::{
    blocking_mutex::raw::NoopRawMutex,
    mutex::{Mutex, MutexGuard},
};
use embassy_time::{Duration, Instant, Timer};
use zenoh_proto::{EitherError, TransportLinkError, msgs::NetworkMessage};

use crate::{
    config::ZConfig,
    io::transport::{
        TransportLink, TransportLinkRx, TransportLinkTx, ZTransportLinkRx, ZTransportLinkTx,
    },
    platform::{ZLink, ZLinkManager},
};

type Link<'res, Config> = <<Config as ZConfig>::LinkManager as ZLinkManager>::Link<'res>;
type LinkTx<'res, Config> =
    <<<Config as ZConfig>::LinkManager as ZLinkManager>::Link<'res> as ZLink>::Tx<'res>;
type LinkRx<'res, Config> =
    <<<Config as ZConfig>::LinkManager as ZLinkManager>::Link<'res> as ZLink>::Rx<'res>;

pub(crate) struct Driver<'res, Config>
where
    Config: ZConfig + 'res,
{
    tx: Mutex<NoopRawMutex, TransportLinkTx<'res, LinkTx<'res, Config>, Config::Buff>>,
    rx: RefCell<TransportLinkRx<'res, LinkRx<'res, Config>, Config::Buff>>,
}

impl<'res, Config> Driver<'res, Config>
where
    Config: ZConfig,
{
    pub fn new(transport: &'res mut TransportLink<Link<'res, Config>, Config::Buff>) -> Self {
        let (tx, rx) = transport.split();

        Self {
            tx: Mutex::new(tx),
            rx: RefCell::new(rx),
        }
    }

    pub async fn tx(
        &self,
    ) -> MutexGuard<'_, NoopRawMutex, TransportLinkTx<'res, LinkTx<'res, Config>, Config::Buff>>
    {
        self.tx.lock().await
    }

    pub async fn run<State, E, Update>(
        &self,
        state: &Mutex<NoopRawMutex, State>,
        mut update: Update,
    ) -> core::result::Result<(), EitherError<TransportLinkError, E>>
    where
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
        rx: &mut TransportLinkRx<'res, LinkRx<'res, Config>, Config::Buff>,
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
