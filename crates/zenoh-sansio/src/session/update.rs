use core::{iter::Take, time::Duration};

use zenoh_proto::{
    ZResult,
    transport::{TransportBatch, TransportBody},
};

use crate::{
    Session, ZSessionError, establish,
    event::{AsEventAccumulator, Event, EventInner},
    session::SessionState,
};

impl Session {
    pub fn update<'a, T>(
        &mut self,
        mut bytes: &'a [u8],
        time: Duration,
        mut events: T,
    ) -> crate::ZResult<
        ZResult<
            impl Iterator<Item = Event<'a>> + use<'a, T>,
            impl Iterator<Item = Event<'a>> + use<'a, T>,
        >,
        ZSessionError,
    >
    where
        T: AsEventAccumulator<'a> + IntoIterator<Item = Event<'a>>,
    {
        let mut acc = events.as_accumulator();
        let mut n = 0;

        macro_rules! push {
            ($event:expr) => {
                match <_ as super::event::EventAccumulator>::push(&mut acc, $event) {
                    Ok(_) => {
                        n += 1;
                    }
                    Err(_) => {
                        drop(acc);
                        return Ok(Err(events.into_iter().take(n)));
                    }
                }
            };
        }

        let mut disconnect = None;
        if let SessionState::Connected {
            next_recv_keepalive,
            next_send_keepalive,
            mine,
            other,
            ..
        } = &mut self.state
        {
            if bytes.is_empty() && time >= *next_recv_keepalive {
                disconnect = Some(mine.clone());
            }

            if !bytes.is_empty() {
                *next_recv_keepalive = time + other.other_lease;
            }

            if time >= *next_send_keepalive {
                *next_send_keepalive = time + mine.mine_lease / 4;

                push!(Event {
                    inner: EventInner::KeepAlive
                });
            }
        }

        let mut batch = TransportBatch::new(&mut bytes);
        while let Some(msg) = batch.next() {
            match msg? {
                TransportBody::Close(_) => {
                    if let SessionState::Connected { mine, .. } = &self.state {
                        disconnect = Some(mine.clone());
                    }
                }
                TransportBody::InitAck(ack) => {
                    let Session { state } = self;

                    if let SessionState::Disconnected { mine } = state {
                        let (s, event) = establish::handle_init_ack(mine.clone(), ack)?;

                        *state = s;
                        push!(event);
                    }
                }
                TransportBody::OpenAck(ack) => {
                    let Session { state } = self;

                    if let SessionState::Connecting {
                        mine,
                        negotiated,
                        other_zid,
                    } = state
                    {
                        let s = establish::handle_open_ack(
                            time,
                            mine.clone(),
                            negotiated.clone(),
                            other_zid.clone(),
                            ack,
                        );

                        *state = s;
                    }
                }
                TransportBody::KeepAlive(_) => {}
                _ => {
                    zenoh_proto::debug!("Ignoring unexpected transport message");
                }
            }
        }

        if let Some(mine) = disconnect {
            self.state = SessionState::Disconnected { mine };
        }

        drop(acc);
        Ok(Ok::<_, Take<<T as IntoIterator>::IntoIter>>(
            events.into_iter().take(n),
        ))
    }
}
