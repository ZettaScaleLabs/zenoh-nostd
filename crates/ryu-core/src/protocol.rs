pub mod core;

pub mod network;
pub mod transport;
pub mod zenoh;

crate::__internal_err! {
    /// Errors related to IO operations on byte buffers
    #[err = "protocol error"]
    enum ProtocolError {
        CouldNotParse
    }
}
