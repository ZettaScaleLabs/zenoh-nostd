use zenoh_proto::{exts::Value, msgs::*, *};

use crate::{
    Sample,
    api::{
        ZConfig,
        callbacks::{ZCallbacks, ZDynCallback},
        resources::SessionResources,
    },
};

impl<'res, Config> super::Driver<'res, Config>
where
    Config: ZConfig,
{
    pub(crate) async fn update<'a>(
        &self,
        msgs: impl Iterator<Item = (NetworkMessage<'a>, &'a [u8])>,
        resources: &SessionResources<'res, Config>,
    ) -> crate::ZResult<()> {
        for msg in msgs {
            match msg.0.body {
                NetworkBody::Push(Push {
                    wire_expr,
                    payload: PushBody::Put(Put { payload, .. }),
                    ..
                }) => {
                    let ke = wire_expr.suffix;
                    let ke = keyexpr::new(ke)?;
                    let sample = Sample::new(ke, payload);

                    let mut sub_cb = resources.sub_callbacks.lock().await;
                    for cb in sub_cb.intersects(ke) {
                        cb.call(&sample).await;
                    }
                }
                NetworkBody::Response(Response {
                    rid,
                    wire_expr,
                    payload,
                    ..
                }) => {
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
                NetworkBody::ResponseFinal(ResponseFinal { rid, .. }) => {
                    let mut res = resources.get_callbacks.lock().await;
                    res.remove(rid)?;

                    // TODO: also close channels
                }
                NetworkBody::Request(Request {
                    id,
                    wire_expr,
                    payload:
                        RequestBody::Query(Query {
                            parameters, body, ..
                        }),
                    ..
                }) => {
                    let ke = wire_expr.suffix;
                    let ke = keyexpr::new(ke)?;
                    let query = crate::api::Query::new(
                        self,
                        resources,
                        id,
                        ke,
                        if parameters.is_empty() {
                            None
                        } else {
                            Some(parameters)
                        },
                        match body {
                            Some(Value { payload, .. }) => Some(payload),
                            None => None,
                        },
                    );

                    let mut queryable_cb = resources.queryable_callbacks.lock().await;
                    let count = queryable_cb.intersects(ke).count();
                    queryable_cb.set_counter(id, count)?;
                    for cb in queryable_cb.intersects(ke) {
                        cb.call(&query).await;
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }
}
