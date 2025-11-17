use crate::zenoh::{err::*, put::*, query::*, reply::*, *};

const NUM_ITER: usize = 100;
const MAX_PAYLOAD_SIZE: usize = 512;

super::roundtrips!(ext, zenoh, EntityGlobalId, SourceInfo, Value, Attachment);
super::roundtrips!(
    zenoh,
    Err,
    Put,
    Query,
    Reply,
    PushBody,
    RequestBody,
    ResponseBody
);
