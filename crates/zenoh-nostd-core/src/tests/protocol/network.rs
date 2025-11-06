use crate::network::*;

const NUM_ITER: usize = 100;
const MAX_PAYLOAD_SIZE: usize = 512;

crate::roundtrips!(ext, network, QoS, NodeId);
crate::roundtrips!(network,);
