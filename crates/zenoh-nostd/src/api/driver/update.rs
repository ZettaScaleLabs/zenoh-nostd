use zenoh_proto::{msgs::*, *};

use crate::{
    Sample,
    api::{
        ZConfig,
        callbacks::{ZCallbacks, ZDynCallback},
        resources::SessionResources,
    },
};

impl<'transport, Config> super::Driver<'transport, Config>
where
    Config: ZConfig,
{
    pub(crate) async fn update<'res>(
        &self,
        reader: &[u8],
        resources: &SessionResources<'res, Config>,
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

                    let mut sub_cb = resources.sub_callbacks.lock().await;
                    for cb in sub_cb.intersects(ke) {
                        cb.call(&sample).await;
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
                    let ke = wire_expr.suffix;
                    let ke = keyexpr::new(ke)?;
                    let response = match payload {
                        ResponseBody::Reply(Reply {
                            payload: PushBody::Put(Put { payload, .. }),
                            ..
                        }) => crate::Response::Ok(crate::Sample::new(ke, payload)),
                        ResponseBody::Err(Err { payload, .. }) => {
                            crate::Response::Err(crate::Sample::new(ke, payload))
                        }
                    };

                    let mut get_cb = resources.get_callbacks.lock().await;
                    if let Some(cb) = get_cb.get(rid) {
                        cb.call(&response).await;
                    }
                }
                Message::ResponseFinal {
                    body: ResponseFinal { rid, .. },
                    ..
                } => {
                    let mut res = resources.get_callbacks.lock().await;
                    res.remove(rid)?;
                }
                // Message::Request {
                //     body:
                //         Request {
                //             id,
                //             wire_expr,
                //             payload:
                //                 RequestBody::Query(Query {
                //                     parameters, body, ..
                //                 }),
                //             ..
                //         },
                //     ..
                // } => {
                //     let ke = wire_expr.suffix;
                //     let ke = keyexpr::new(ke)?;
                //     let query = crate::api::Query::new(
                //         self,
                //         id,
                //         ke,
                //         if parameters.is_empty() {
                //             None
                //         } else {
                //             Some(parameters)
                //         },
                //         match body {
                //             Some(Value { payload, .. }) => Some(payload),
                //             None => None,
                //         },
                //     );

                //     let mut queryable_cb = resources.queryable_callbacks.lock().await;
                //     for cb in queryable_cb.intersects(ke) {
                //         cb.call(&query).await;
                //     }

                //     let queryable_ch = &resources.queryable_channels;
                //     let guard = queryable_ch.lock().await;
                //     for ch in queryable_ch.intersects(&guard, ke).await {
                //         ch.send(&query).await?;
                //     }
                // }
                _ => {}
            }
        }

        Ok(())
    }
}
