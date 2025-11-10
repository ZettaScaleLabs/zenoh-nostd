use crate::zenoh::{err::*, put::*, query::*, reply::*, *};

const NUM_ITER: usize = 100;
const MAX_PAYLOAD_SIZE: usize = 512;

crate::roundtrips!(ext, zenoh, EntityGlobalId, SourceInfo, Value, Attachment);
crate::roundtrips!(
    zenoh,
    Err,
    Put,
    Query,
    Reply,
    PushBody,
    RequestBody,
    ResponseBody
);
