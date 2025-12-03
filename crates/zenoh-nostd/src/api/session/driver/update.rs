use zenoh_proto::{
    BatchReader, Message, keyexpr,
    msgs::{Push, PushBody},
};

use ::core::ops::DerefMut;

use crate::{
    api::{Sample, SessionResources},
    platform::ZPlatform,
};

impl<'a, Platform, TxBux, RxBux> super::Driver<'a, Platform, TxBux, RxBux>
where
    Platform: ZPlatform,
{
    pub async fn update<
        'b,
        const MAX_KEYEXPR_LEN: usize,
        const MAX_PARAMETERS_LEN: usize,
        const MAX_PAYLOAD_LEN: usize,
        const MAX_QUEUED: usize,
        const MAX_CALLBACKS: usize,
        const MAX_SUBSCRIBERS: usize,
    >(
        &self,
        reader: &'b [u8],
        resources: &SessionResources<
            MAX_KEYEXPR_LEN,
            MAX_PARAMETERS_LEN,
            MAX_PAYLOAD_LEN,
            MAX_QUEUED,
            MAX_CALLBACKS,
            MAX_SUBSCRIBERS,
        >,
    ) -> crate::ZResult<()> {
        let batch = BatchReader::new(reader);

        for msg in batch {
            match msg {
                Message::KeepAlive(_) => {
                    zenoh_proto::trace!("Received KeepAlive");
                }
                Message::Push {
                    body:
                        Push {
                            wire_expr,
                            payload: PushBody::Put(put),
                            ..
                        },
                    ..
                } => {
                    let zbuf: &'b [u8] = put.payload;

                    let ke: &'b str = wire_expr.suffix;
                    let ke: &'b keyexpr = keyexpr::new(ke)?;

                    let mut cb_guard = resources.subscribers.lock().await;
                    let cb = cb_guard.deref_mut();

                    let matching_callbacks = cb
                        .iter()
                        .filter_map(|(k, v)| if k.intersects(ke) { Some(v) } else { None })
                        .filter_map(|cb_id| resources.callbacks.get(cb_id));

                    for callback in matching_callbacks {
                        let sample = Sample::new(ke, zbuf);
                        callback.call_subscriber(sample).await?;
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }
}
