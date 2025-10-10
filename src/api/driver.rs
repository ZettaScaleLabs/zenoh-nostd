use core::ops::{Deref, DerefMut};

use embassy_futures::select::select;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embassy_time::{Instant, Timer};

use crate::{
    api::{
        sample::ZSample,
        subscriber::{ZCallback, ZCallbackMap},
    },
    io::transport::{TransportConfig, TransportRx, TransportTx},
    keyexpr::{
        borrowed::keyexpr,
        intersect::{DEFAULT_INTERSECTOR, Intersector},
    },
    platform::{Platform, ZCommunicationError},
    protocol::{
        core::wire_expr::WireExpr,
        network::{NetworkBody, NetworkMessage},
        transport::{
            self, TransportMessage, TransportSn,
            frame::{Frame, FrameHeader},
            init::{InitAck, InitSyn},
            keepalive::KeepAlive,
            open::{OpenAck, OpenSyn},
        },
        zenoh::PushBody,
    },
    result::ZResult,
    zbuf::{ZBuf, ZBufReader},
};

pub struct TxState<T: Platform> {
    tx_zbuf: &'static mut [u8],
    tx: TransportTx<T>,
    sn: TransportSn,

    next_keepalive: Instant,
}

pub struct RxState<T: Platform> {
    rx_zbuf: &'static mut [u8],
    rx: TransportRx<T>,
}

pub struct CallbackState {
    callbacks: &'static mut dyn ZCallbackMap,
}

pub struct SessionDriver<T: Platform> {
    config: TransportConfig,

    tx: Mutex<CriticalSectionRawMutex, TxState<T>>,
    rx: Mutex<CriticalSectionRawMutex, RxState<T>>,
    callbacks: Mutex<CriticalSectionRawMutex, CallbackState>,
}

impl<T: Platform> SessionDriver<T> {
    pub fn new(
        config: TransportConfig,
        tx: (&'static mut [u8], TransportTx<T>),
        rx: (&'static mut [u8], TransportRx<T>),
        callbacks: &'static mut dyn ZCallbackMap,
    ) -> SessionDriver<T> {
        SessionDriver {
            tx: Mutex::new(TxState {
                tx_zbuf: tx.0,
                tx: tx.1,
                sn: config.negociated_config.mine_sn,
                next_keepalive: Instant::now(),
            }),
            rx: Mutex::new(RxState {
                rx_zbuf: rx.0,
                rx: rx.1,
            }),
            callbacks: Mutex::new(CallbackState { callbacks }),
            config,
        }
    }

    pub async fn internal_update<'a>(
        &self,
        mut reader: ZBufReader<'a>,
    ) -> ZResult<(), ZCommunicationError> {
        TransportMessage::decode_batch_async(
            &mut reader,
            None::<fn(InitSyn) -> ZResult<()>>,
            None::<fn(InitAck) -> ZResult<()>>,
            None::<fn(OpenSyn) -> ZResult<()>>,
            None::<fn(OpenAck) -> ZResult<()>>,
            Some(|| {
                crate::trace!("Received KeepAlive");

                Ok(())
            }),
            Some(async |_: &FrameHeader, msg: NetworkMessage<'a>| {
                if let NetworkBody::Push(push) = msg.body {
                    match push.payload {
                        PushBody::Put(put) => {
                            let zbuf: ZBuf<'a> = put.payload;

                            let wke: &keyexpr = push.wire_expr.as_str().try_into().unwrap();

                            let mut cb_guard = self.callbacks.lock().await;
                            let cb = cb_guard.deref_mut();

                            let matching_callbacks = cb.callbacks.iter().filter_map(|(k, v)| {
                                let ke = k.as_str().try_into().unwrap();
                                if DEFAULT_INTERSECTOR.intersect(ke, wke) {
                                    Some(v)
                                } else {
                                    None
                                }
                            });

                            for callback in matching_callbacks {
                                let sample: ZSample<'a> =
                                    ZSample::new(push.wire_expr.clone(), zbuf);

                                callback.call(&sample).await;
                            }
                        }
                    }
                }

                Ok(())
            }),
        )
        .await?;

        Ok(())
    }

    pub async fn register_callback(
        &self,
        expr: WireExpr<'static>,
        callback: ZCallback,
    ) -> ZResult<()> {
        let mut cb_guard = self.callbacks.lock().await;
        let cb = cb_guard.deref_mut();

        cb.callbacks.insert_callback(expr, callback).map(|_| ())
    }

    pub async fn send(&self, msg: NetworkMessage<'_>) -> ZResult<(), ZCommunicationError> {
        let mut tx_guard = self.tx.lock().await;
        let tx = tx_guard.deref_mut();

        let frame = Frame {
            reliability: msg.reliability,
            sn: tx.sn,
            ext_qos: transport::frame::ext::QoSType::DEFAULT,
            payload: &[msg],
        };
        let tmsg = TransportMessage {
            body: transport::TransportBody::Frame(frame),
        };

        tx.tx.send(tx.tx_zbuf, &tmsg).await?;

        tx.sn = tx.sn.wrapping_add(1); // TODO wrap around Resolution
        tx.next_keepalive = Instant::now()
            + (self.config.mine_config.mine_lease / (self.config.mine_config.keep_alive as u32))
                .try_into()
                .unwrap();

        Ok(())
    }

    pub async fn run(&self) -> ZResult<(), ZCommunicationError> {
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
                            crate::warn!("Connection closed by peer");
                            break Err(ZCommunicationError::ConnectionClosed);
                        }
                        embassy_futures::select::Either::Second(tmsg) => match tmsg {
                            Ok(tmsg) => {
                                self.internal_update(tmsg).await?;
                            }
                            Err(_) => {
                                crate::warn!("Did not read from connection");
                                break Err(ZCommunicationError::DidNotRead);
                            }
                        },
                    }
                }
                embassy_futures::select::Either::Second(_) => {
                    let mut tx_guard = self.tx.lock().await;
                    let tx = tx_guard.deref_mut();
                    if Instant::now() >= tx.next_keepalive {
                        let tmsg = TransportMessage {
                            body: transport::TransportBody::KeepAlive(KeepAlive),
                        };
                        crate::trace!("Sending KeepAlive");
                        tx.tx.send(tx.tx_zbuf, &tmsg).await?;
                        tx.next_keepalive = Instant::now() + keep_alive_timeout.try_into().unwrap();
                    }
                }
            }
        }
    }
}
