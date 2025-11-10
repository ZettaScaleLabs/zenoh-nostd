use crate::network::{declare::*, interest::*, push::*, request::*, response::*, *};

const NUM_ITER: usize = 100;
const MAX_PAYLOAD_SIZE: usize = 512;

crate::roundtrips!(
    ext,
    network,
    QoS,
    NodeId,
    QueryTarget,
    Budget,
    Timeout,
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
