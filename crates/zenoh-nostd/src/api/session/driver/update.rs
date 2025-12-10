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
                            payload: PushBody::Put(Put { payload, .. }),
                            ..
                        },
                    ..
                } => {
                    let ke = wire_expr.suffix;
                    let ke = keyexpr::new(ke)?;
                    let sample = Sample::new(ke, payload);

                    let sub_cb = resources.sub_callbacks.lock().await;
                    for cb in sub_cb.intersects(ke) {
                        cb.execute(&sample).await;
                    }

                    let sub_ch = &resources.sub_channels;
                    let guard = sub_ch.lock().await;
                    for ch in sub_ch.intersects(&guard, ke).await {
                        ch.send(&sample).await?;
                    }
                }
                Message::Response {
                    body:
                        Response {
                            rid,
                            wire_expr,
                            payload,
                            ..
                        },
                    ..
                } => {
                    let (is_ok, payload) = match payload {
                        ResponseBody::Reply(Reply {
                            payload: PushBody::Put(Put { payload, .. }),
                            ..
                        }) => (true, payload),
                        ResponseBody::Err(Err { payload, .. }) => (false, payload),
                    };

                    let ke = wire_expr.suffix;
                    let ke = keyexpr::new(ke)?;
                    let response = crate::api::Response::new(is_ok, &ke, payload);

                    let get_cb = resources.get_callbacks.lock().await;
                    if let Some(cb) = get_cb.get(rid) {
                        cb.execute(&response).await;
                    }

                    let get_ch = &resources.get_channels;
                    let guard = get_ch.lock().await;
                    if let Some(ch) = get_ch.get(&guard, rid).await {
                        ch.send(&response).await?;
                    }
                }
                Message::ResponseFinal {
                    body: ResponseFinal { rid, .. },
                    ..
                } => {
                    resources.get_callbacks.lock().await.remove(rid)?;
                    resources.get_channels.remove(rid).await?;
                }
                _ => {}
            }
        }

        Ok(())
    }
}
