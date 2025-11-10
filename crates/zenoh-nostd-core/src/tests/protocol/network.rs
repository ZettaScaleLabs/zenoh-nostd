use core::time::Duration;

use rand::{Rng, thread_rng};

use crate::{
    ZWriter,
    network::{declare::*, interest::*, push::*, request::*, response::*, *},
};

const NUM_ITER: usize = 100;
const MAX_PAYLOAD_SIZE: usize = 512;

crate::roundtrips!(
    ext,
    network,
    QoS,
    NodeId,
    QueryTarget,
    Budget,
    Duration,
    QueryableInfo
);
crate::roundtrips!(
    network,
    DeclareKeyExpr,
    UndeclareKeyExpr,
    DeclareSubscriber,
    UndeclareSubscriber,
    DeclareQueryable,
    UndeclareQueryable,
    DeclareToken,
    UndeclareToken,
    DeclareFinal,
    DeclareBody,
    Declare,
    Interest,
    Push,
    Request,
    Response,
    ResponseFinal
);

trait RandDuration {
    fn rand(w: &mut ZWriter) -> Self;
}

impl RandDuration for Duration {
    fn rand(_: &mut ZWriter) -> Self {
        Duration::from_millis(thread_rng().gen_range(0..10_000))
    }
}
