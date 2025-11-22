use core::ops::DerefMut;

use zenoh_proto::{
    ZResult, keyexpr,
    network::NetworkBody,
    transport::{TransportBatch, TransportBody},
    zenoh::{PushBody, RequestBody, ResponseBody},
};

use crate::{
    ZQuery, ZReply,
    api::{driver::SessionDriver, sample::ZSample},
    platform::Platform,
};

impl<T: Platform> SessionDriver<T> {
    pub(crate) async fn internal_update<'a>(&'static self, reader: &'a [u8]) -> ZResult<()> {
        let mut batch = TransportBatch::new(reader);

        while let Some(msg) = batch.next() {
            match msg? {
                TransportBody::KeepAlive(_) => {
                    zenoh_proto::trace!("Received KeepAlive");
                }

                TransportBody::Frame(mut frame) => {
                    for msg in frame.msgs.by_ref() {
                        match msg? {
                            NetworkBody::Push(push) => match push.payload {
                                PushBody::Put(put) => {
                                    let zbuf: &'a [u8] = put.payload;

                                    let wke: &'a str = push.wire_expr.suffix;
                                    let wke: &'a keyexpr = keyexpr::new(wke)?;

                                    let mut cb_guard = self.subscribers.lock().await;
                                    let cb = cb_guard.deref_mut();

                                    let matching_callbacks =
                                        cb.callbacks.iter().filter_map(|(k, v)| {
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
                            },
                            NetworkBody::Response(resp) => {
                                let rid = resp.rid;

                                let wke: &'a str = resp.wire_expr.suffix;
                                let wke: &'a keyexpr = keyexpr::new(wke)?;

                                let mut cb_guard = self.replies.lock().await;
                                let cb = cb_guard.deref_mut();

                                cb.callbacks.drop_timedout();
                                let matching_callbacks = cb
                                    .callbacks
                                    .iter()
                                    .filter_map(|(k, v)| if *k == rid { Some(v) } else { None });

                                for callback in matching_callbacks {
                                    let reply = match &resp.payload {
                                        ResponseBody::Reply(reply) => match &reply.payload {
                                            PushBody::Put(put) => {
                                                let sample = ZSample::new(wke, put.payload);

                                                ZReply::Ok(sample)
                                            }
                                        },
                                        ResponseBody::Err(err) => {
                                            let sample = ZSample::new(wke, err.payload);
                                            ZReply::Err(sample)
                                        }
                                    };

                                    callback.call(reply);
                                }
                            }
                            NetworkBody::ResponseFinal(resp) => {
                                let rid = resp.rid;

                                let mut cb_guard = self.replies.lock().await;
                                let cb = cb_guard.deref_mut();

                                cb.callbacks.remove(&rid);
                            }
                            NetworkBody::Request(request) => match request.payload {
                                RequestBody::Query(query) => {
                                    let rid = request.id;

                                    let ke: &'a str = request.wire_expr.suffix;
                                    let ke: &'a keyexpr = keyexpr::new(ke)?;

                                    let mut cb_guard = self.queryables.lock().await;
                                    let cb = cb_guard.deref_mut();

                                    let matching_callbacks =
                                        cb.callbacks.iter().filter_map(|(k, v)| {
                                            if cb.callbacks.intersects(k, ke) {
                                                Some(v)
                                            } else {
                                                None
                                            }
                                        });

                                    for callback in matching_callbacks {
                                        let query = ZQuery::new(
                                            rid,
                                            self,
                                            ke,
                                            match query.parameters {
                                                "" => None,
                                                p => Some(p),
                                            },
                                            match &query.body {
                                                None => None,
                                                Some(v) => Some(v.payload),
                                            },
                                        );

                                        callback.call(query).await?;
                                    }
                                }
                            },
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }
}
