use core::ops::DerefMut;

use crate::{
    api::{driver::SessionDriver, sample::ZSample},
    keyexpr::borrowed::keyexpr,
    platform::{Platform, ZCommunicationError},
    protocol::{
        network::{NetworkBody, NetworkMessage},
        transport::{
            TransportMessage,
            frame::FrameHeader,
            init::{InitAck, InitSyn},
            open::{OpenAck, OpenSyn},
        },
        zenoh::PushBody,
    },
    result::ZResult,
    zbuf::{ZBuf, ZBufReader},
};

impl<T: Platform> SessionDriver<T> {
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

                            let wke: &'a str = push.wire_expr.suffix;
                            let wke: &'a keyexpr = wke.try_into().unwrap();

                            let mut cb_guard = self.subscribers.lock().await;
                            let cb = cb_guard.deref_mut();

                            let matching_callbacks = cb.callbacks.iter().filter_map(|(k, v)| {
                                if cb.callbacks.intersects(k, wke) {
                                    Some(v)
                                } else {
                                    None
                                }
                            });

                            for callback in matching_callbacks {
                                let sample: ZSample<'a> = ZSample::new(wke, zbuf);

                                callback.call(sample).await?;
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
}
