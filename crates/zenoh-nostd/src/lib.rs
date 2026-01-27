#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

// mod api;

// mod config;
mod io;
// mod resources;

// pub mod session {
//     pub use super::config::ZSessionConfig;
//     pub use super::io::transport::TransportLinkManager;
//     pub use super::resources::SessionResources;
//     pub use zenoh_proto::{Endpoint, Error};

//     pub use super::api::{
//         response::*,
//         sample::*,
//         session::{get::*, r#pub::*, put::*, querier::*, sub::*},
//     };

//     pub mod zenoh {
//         pub use super::super::api::callbacks::storage;
//         pub use super::super::api::session::{
//             Session, session_connect as connect,
//             session_connect_ignore_invalid_sn as connect_ignore_invalid_sn,
//             session_listen as listen, session_listen_ignore_invalid_sn as listen_ignore_invalid_sn,
//         };

//         pub use crate::{__session_connect as connect, __session_listen as listen};

//         pub use zenoh_proto::{debug, error, info, keyexpr, trace, warn, zbail};

//         pub type ZResult<T> = core::result::Result<T, super::Error>;
//     }
// }

pub mod platform {
    pub use super::io::link::{ZLink, ZLinkInfo, ZLinkManager, ZLinkRx, ZLinkTx};
    pub use zenoh_derive::{ZLink, ZLinkInfo, ZLinkRx, ZLinkTx};
    pub use zenoh_proto::{Endpoint, LinkError};

    pub mod zenoh {
        pub use zenoh_proto::{debug, error, info, trace, warn, zbail};
    }
}
