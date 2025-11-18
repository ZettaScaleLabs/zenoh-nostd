use core::time::Duration;

use zenoh_proto::{Resolution, ZenohIdProto};

enum SessionState {
    Disconnected {
        mine: MineConfig,
    },
    Connecting {
        mine: MineConfig,
        negotiated: NegotiatedConfig,

        other_zid: ZenohIdProto,
    },
    Connected {
        mine: MineConfig,
        other: OtherConfig,
        negotiated: NegotiatedConfig,

        next_recv_keepalive: Duration,
        next_send_keepalive: Duration,
    },
}

struct MineConfig {
    mine_zid: ZenohIdProto,
    mine_resolution: Resolution,
    mine_batch_size: u16,

    mine_lease: Duration,
}

struct OtherConfig {
    other_zid: ZenohIdProto,
    other_sn: u32,
    other_lease: Duration,
}

struct NegotiatedConfig {
    negotiated_sn: u32,
    negotiated_resolution: Resolution,
    negotiated_batch_size: u16,
}

pub struct Session {
    state: SessionState,
}
