use zenoh_protocol::{
    core::{resolution::Resolution, whatami::WhatAmI, ZenohIdProto},
    transport::{init::ext, BatchSize, InitSyn},
    VERSION,
};

fn main() {
    let buffer = [0u8; 128];
    let ext_autch = ext::Auth::new(&buffer);

    let init_syn = InitSyn {
        version: VERSION,
        whatami: WhatAmI::Client,
        zid: ZenohIdProto::default(),
        resolution: Resolution::default(),
        batch_size: BatchSize::MAX,
        ext_qos: None,
        ext_qos_link: None,
        ext_auth: Some(ext_autch),
        ext_mlink: None,
        ext_lowlatency: None,
        ext_compression: None,
        ext_patch: ext::PatchType::NONE,
    };
}
