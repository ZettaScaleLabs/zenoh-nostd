use crate::transport::{close::*, frame::*, init::*, keepalive::*, open::*, *};

const NUM_ITER: usize = 100;
const MAX_PAYLOAD_SIZE: usize = 512;

crate::roundtrips!(ext, transport, QoSLink, Auth, MultiLink, PatchType);
crate::roundtrips!(
    transport,
    Close,
    FrameHeader,
    InitSyn,
    InitAck,
    KeepAlive,
    OpenSyn,
    OpenAck
);
