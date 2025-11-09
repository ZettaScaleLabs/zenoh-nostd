use crate::network::{push::*, request::*, response::*, *};

const NUM_ITER: usize = 100;
const MAX_PAYLOAD_SIZE: usize = 512;

crate::roundtrips!(ext, network, QoS, NodeId, QueryTarget, Budget, Timeout);
crate::roundtrips!(network, Push, Request, Response, ResponseFinal);
