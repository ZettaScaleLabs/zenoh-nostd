#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

// pub(crate) mod api;
pub(crate) mod io;

// pub mod session {
//     // Callbacks
//     pub use super::api::{
//         FixedCapacityGetCallbacks, FixedCapacityQueryableCallbacks, FixedCapacitySubCallbacks,
//         storage::{Raw, RawOrBox},
//     };

//     // Primitives
//     pub use super::api::{OwnedQuery, OwnedResponse, OwnedSample, Query, Sample};

//     // Objects
//     pub use super::api::{Resources, Response, Session};

//     // Traits
//     pub use super::api::{ZConfig, ZResult, ZTransportConfig};

//     // Reexport
//     pub use zenoh_proto::{EndPoint, fields::Encoding, keyexpr};
// }

pub mod broker {}

pub mod platform {
    pub use super::io::{ZLink, ZLinkInfo, ZLinkRx, ZLinkTx};
}
