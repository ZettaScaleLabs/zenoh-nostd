use core::ops::DerefMut;

use zenoh_proto::{
    ZResult, keyexpr,
    network::NetworkBody,
    transport::{TransportBatch, TransportBody},
    zenoh::{PushBody, ResponseBody},
};

use crate::{
    ZReply,
    api::{driver::SessionDriver, sample::ZSample},
    platform::Platform,
};

impl<T: Platform> SessionDriver<T> {
    pub(crate) async fn internal_update<'a>(&self, mut reader: &'a [u8]) -> ZResult<()> {
        let mut batch = TransportBatch::new(&mut reader);

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

                                let mut cb_guard = self.queries.lock().await;
                                let cb = cb_guard.deref_mut();

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

                                    callback.call(reply).await?;
                                }
                            }
                            NetworkBody::ResponseFinal(resp) => {
                                let rid = resp.rid;

                                let mut cb_guard = self.queries.lock().await;
                                let cb = cb_guard.deref_mut();

                                cb.callbacks.remove(&rid);
                            }
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
