#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod config;
mod io;
mod resources;

pub mod session {
    pub use super::config::ZSessionConfig;
    pub use super::io::TransportLinkManager;
    pub use super::resources::{Session, SessionResources};
    pub use zenoh_proto::{Endpoint, Error, debug, error, info, trace, warn, zbail};

    pub mod zenoh {
        pub use super::super::resources::{session_connect as connect, session_listen as listen};
    }
}

pub mod platform {
    pub use super::io::{Link, ZLink, ZLinkInfo, ZLinkManager, ZLinkRx, ZLinkTx};
    pub use zenoh_proto::{LinkError, debug, error, info, trace, warn, zbail};
}
