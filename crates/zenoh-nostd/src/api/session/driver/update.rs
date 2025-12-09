use core::ops::Deref;
use zenoh_proto::{Message, msgs::*, *};

use crate::api::{Sample, SessionResources, ZCallback, ZCallbacks, ZDriverConfig, ZSessionConfig};

impl<Config> super::Driver<'_, Config>
where
    Config: ZDriverConfig + ZSessionConfig,
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

                    let sub_guard = resources.subscribers.lock().await;
                    let subscribers = sub_guard.deref();

                    for callback in subscribers.intersects(ke) {
                        callback.execute(&sample).await;
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }
}
