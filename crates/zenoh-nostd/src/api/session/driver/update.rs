use zenoh_proto::{Message, msgs::*, *};

use crate::api::{Sample, SessionResources, ZCallback, ZCallbacks, ZChannel, ZChannels, ZConfig};

impl<Config> super::Driver<'_, Config>
where
    Config: ZConfig,
{
    pub async fn update(
        &self,
        reader: &[u8],
        resources: &SessionResources<Config>,
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
                    let payload = put.payload;
                    let ke = wire_expr.suffix;
                    let ke = keyexpr::new(ke)?;
                    let sample = Sample::new(ke, payload);

                    let sub_cb = resources.sub_callbacks.lock().await;
                    for callback in sub_cb.intersects(ke) {
                        callback.execute(&sample).await;
                    }

                    let sub_ch = &resources.sub_channels;
                    let guard = sub_ch.lock().await;
                    for ch in sub_ch.intersects(&guard, ke).await {
                        ch.send(&sample).await?;
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }
}
